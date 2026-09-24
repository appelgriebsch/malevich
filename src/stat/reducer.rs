//! The shared reducer vocabulary: one set of names for every aggregating stat.

/// A named aggregation — how a set of values collapses to one number. The one
/// vocabulary shared by [`Agg`](super::Agg), [`Window`](super::Window), and
/// binned reduction ([`binned`](super::binned)), following the Observable Plot
/// convention.
///
/// Non-finite values are excluded before reducing (the gap convention). An
/// empty set reduces to `0` for [`Count`](Reducer::Count) and
/// [`Sum`](Reducer::Sum) — real answers — and to a gap (`NaN`) for everything
/// else; the spread reducers need two values before they answer.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Reducer {
    /// The number of finite values.
    Count,
    /// Their sum.
    Sum,
    /// Their mean.
    Mean,
    /// Their median — [`Percentile`](Reducer::Percentile) at 0.5.
    Median,
    /// The smallest.
    Min,
    /// The largest.
    Max,
    /// The type-7 quantile at a position in `[0, 1]` — the same estimator the
    /// box plot's quartiles use (the R default).
    Percentile(f64),
    /// The sample standard deviation (`n − 1` in the denominator, as pandas and
    /// R report it); a gap below two values.
    Deviation,
    /// The sample variance (`n − 1`); a gap below two values.
    Variance,
    /// The standard error of the mean, `Deviation / √n`; a gap below two
    /// values.
    StdErr,
    /// The first finite value, in collection order.
    First,
    /// The last finite value, in collection order.
    Last,
}

impl Reducer {
    /// Reduces `values`, excluding non-finite members.
    ///
    /// # Panics
    ///
    /// Panics when a [`Percentile`](Reducer::Percentile) position is not in
    /// `[0, 1]`.
    pub fn reduce(&self, values: &[f64]) -> f64 {
        let mut state = ReducerState::new(*self);
        if let ReducerState::Quantile { values: finite, .. } = &mut state {
            finite.reserve(values.len());
        }
        for &value in values {
            state.add(value);
        }
        state.finish()
    }
}

/// The execution state behind a [`Reducer`]. Streaming summaries retain only
/// their sufficient statistic; order statistics retain their finite sample.
/// Keeping this private lets bins and windows specialize without turning the
/// public reducer vocabulary into a trait framework.
#[derive(Debug, Clone)]
pub(crate) enum ReducerState {
    Count(usize),
    Sum(f64),
    Mean {
        count: usize,
        mean: f64,
    },
    Min(Option<f64>),
    Max(Option<f64>),
    Quantile {
        position: f64,
        values: Vec<f64>,
    },
    /// Welford's running moments for the three spread reducers.
    Spread {
        kind: Spread,
        count: usize,
        mean: f64,
        m2: f64,
    },
    First(Option<f64>),
    Last(Option<f64>),
}

/// Which spread statistic a [`ReducerState::Spread`] finishes as.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Spread {
    Deviation,
    Variance,
    StdErr,
}

impl ReducerState {
    pub(crate) fn new(reducer: Reducer) -> ReducerState {
        match reducer {
            Reducer::Count => ReducerState::Count(0),
            Reducer::Sum => ReducerState::Sum(0.0),
            Reducer::Mean => ReducerState::Mean {
                count: 0,
                mean: 0.0,
            },
            Reducer::Min => ReducerState::Min(None),
            Reducer::Max => ReducerState::Max(None),
            Reducer::Median => ReducerState::Quantile {
                position: 0.5,
                values: Vec::new(),
            },
            Reducer::Percentile(position) => {
                assert!(
                    (0.0..=1.0).contains(&position),
                    "Reducer::Percentile requires a position in [0, 1]"
                );
                ReducerState::Quantile {
                    position,
                    values: Vec::new(),
                }
            }
            Reducer::Deviation | Reducer::Variance | Reducer::StdErr => ReducerState::Spread {
                kind: match reducer {
                    Reducer::Variance => Spread::Variance,
                    Reducer::StdErr => Spread::StdErr,
                    _ => Spread::Deviation,
                },
                count: 0,
                mean: 0.0,
                m2: 0.0,
            },
            Reducer::First => ReducerState::First(None),
            Reducer::Last => ReducerState::Last(None),
        }
    }

    pub(crate) fn add(&mut self, value: f64) {
        if !value.is_finite() {
            return;
        }
        match self {
            ReducerState::Count(count) => *count += 1,
            ReducerState::Sum(sum) => *sum += value,
            ReducerState::Mean { count, mean } => {
                *count += 1;
                *mean = crate::numeric::lerp(*mean, value, 1.0 / *count as f64);
            }
            ReducerState::Min(minimum) => {
                *minimum = Some(minimum.map_or(value, |current| current.min(value)));
            }
            ReducerState::Max(maximum) => {
                *maximum = Some(maximum.map_or(value, |current| current.max(value)));
            }
            ReducerState::Quantile { values, .. } => values.push(value),
            ReducerState::Spread {
                count, mean, m2, ..
            } => {
                *count += 1;
                let delta = value - *mean;
                *mean += delta / *count as f64;
                *m2 += delta * (value - *mean);
            }
            ReducerState::First(first) => {
                first.get_or_insert(value);
            }
            ReducerState::Last(last) => *last = Some(value),
        }
    }

    pub(crate) fn finish(self) -> f64 {
        match self {
            ReducerState::Count(count) => count as f64,
            ReducerState::Sum(sum) => sum,
            ReducerState::Mean { count: 0, .. } => f64::NAN,
            ReducerState::Mean { mean, .. } => mean,
            ReducerState::Min(minimum) => minimum.unwrap_or(f64::NAN),
            ReducerState::Max(maximum) => maximum.unwrap_or(f64::NAN),
            ReducerState::Quantile { values, .. } if values.is_empty() => f64::NAN,
            ReducerState::Quantile {
                position,
                mut values,
            } => {
                values.sort_by(f64::total_cmp);
                quantile_sorted(&values, position)
            }
            ReducerState::Spread { count, .. } if count < 2 => f64::NAN,
            ReducerState::Spread {
                kind, count, m2, ..
            } => {
                let variance = m2 / (count - 1) as f64;
                match kind {
                    Spread::Variance => variance,
                    Spread::Deviation => variance.sqrt(),
                    Spread::StdErr => (variance / count as f64).sqrt(),
                }
            }
            ReducerState::First(first) => first.unwrap_or(f64::NAN),
            ReducerState::Last(last) => last.unwrap_or(f64::NAN),
        }
    }
}

/// The type-7 quantiles of one sample at several positions, sorting once —
/// the efficient shape for Q–Q plots and multi-quantile summaries. Non-finite
/// values are excluded; an empty sample yields gaps.
///
/// # Panics
///
/// Panics when a position is outside `[0, 1]`.
pub fn quantiles(values: &[f64], positions: &[f64]) -> Vec<f64> {
    assert!(
        positions.iter().all(|p| (0.0..=1.0).contains(p)),
        "quantiles requires positions in [0, 1]"
    );
    let mut finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if finite.is_empty() {
        return vec![f64::NAN; positions.len()];
    }
    finite.sort_by(f64::total_cmp);
    positions
        .iter()
        .map(|&position| quantile_sorted(&finite, position))
        .collect()
}

/// The type-7 quantile of a non-empty slice of finite values, by selection
/// rather than a sort — O(n), the same estimator [`quantile_sorted`] answers
/// from an ordered slice. Reorders `values` in place.
pub(crate) fn quantile_select(values: &mut [f64], p: f64) -> f64 {
    let position = (values.len() - 1) as f64 * p;
    let index = position.floor() as usize;
    let fraction = position - index as f64;
    let (_, &mut at, above) = values.select_nth_unstable_by(index, f64::total_cmp);
    if fraction > 0.0 && !above.is_empty() {
        let next = above.iter().copied().fold(f64::INFINITY, f64::min);
        crate::numeric::lerp(at, next, fraction)
    } else {
        at
    }
}

/// The type-7 quantile of an ascending-sorted, non-empty slice (the R
/// default: linear interpolation of the order statistics).
pub(crate) fn quantile_sorted(sorted: &[f64], p: f64) -> f64 {
    let position = (sorted.len() - 1) as f64 * p;
    let index = position.floor() as usize;
    let fraction = position - index as f64;
    if index + 1 < sorted.len() {
        crate::numeric::lerp(sorted[index], sorted[index + 1], fraction)
    } else {
        sorted[index]
    }
}

#[cfg(test)]
#[path = "tests/reducer_tests.rs"]
mod tests;
