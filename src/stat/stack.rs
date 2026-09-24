//! Stacking: series into cumulative bands.

/// Where a stack sits on its value axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum StackOffset {
    /// From the zero baseline: positive values accumulate upward, negative
    /// values downward, each sign on its own side (the default).
    #[default]
    Zero,
    /// Each position's positive bands scaled to fill `[0, 1]` and its negative
    /// bands `[-1, 0]` — the 100 % stack. A position whose total is zero keeps
    /// zero-height bands, never a gap manufactured by `0 / 0`.
    Normalize,
    /// Bands accumulate in order from zero, then every position shifts so its
    /// stack is centered on zero — the silhouette of a streamgraph.
    Center,
}

/// The order series pile up in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum StackOrder {
    /// As given: the first series at the baseline (the default).
    #[default]
    Input,
    /// By descending total: the largest series at the baseline, so the
    /// biggest band is the flattest.
    Sum,
}

/// Configuration for [`stack_with`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct StackOptions {
    /// Where the stack sits on its axis.
    pub offset: StackOffset,
    /// The order series pile up in.
    pub order: StackOrder,
}

impl StackOptions {
    /// The zero baseline in input order — exactly [`stack`].
    pub const fn new() -> StackOptions {
        StackOptions {
            offset: StackOffset::Zero,
            order: StackOrder::Input,
        }
    }

    /// Sets the offset.
    #[must_use]
    pub const fn offset(mut self, offset: StackOffset) -> StackOptions {
        self.offset = offset;
        self
    }

    /// Sets the stacking order.
    #[must_use]
    pub const fn order(mut self, order: StackOrder) -> StackOptions {
        self.order = order;
        self
    }
}

/// Stacks series into `(low, high)` bands: each series sits on the sum of the ones
/// before it, positive values piling upward from zero and negative values
/// downward, each sign on its own side. Feeds [`crate::mark::Area::between`]
/// layers directly, and [`crate::mark::Bars::base`] with `high − low` as the
/// segment lengths.
///
/// Gaps (`NaN`) contribute zero to the running sum — a missing slice of a stack has
/// no thickness; shorter series are treated as zero-padded. Every band has
/// `low <= high`. [`stack_with`] chooses another offset or order.
pub fn stack(series: &[&[f64]]) -> Vec<(Vec<f64>, Vec<f64>)> {
    stack_with(series, StackOptions::new())
}

/// [`stack`] with a chosen [`StackOffset`] and [`StackOrder`]. The bands come
/// back in input order whatever the stacking order, so they still zip with
/// their series' labels.
pub fn stack_with(series: &[&[f64]], options: StackOptions) -> Vec<(Vec<f64>, Vec<f64>)> {
    let length = series.iter().map(|s| s.len()).max().unwrap_or(0);
    let value_at = |k: usize, index: usize| -> f64 {
        let value = series[k].get(index).copied().unwrap_or(0.0);
        if value.is_finite() { value } else { 0.0 }
    };
    let mut order: Vec<usize> = (0..series.len()).collect();
    if options.order == StackOrder::Sum {
        let totals: Vec<f64> = (0..series.len())
            .map(|k| (0..length).map(|index| value_at(k, index)).sum())
            .collect();
        order.sort_by(|&a, &b| totals[b].total_cmp(&totals[a]).then(a.cmp(&b)));
    }

    let mut bands: Vec<(Vec<f64>, Vec<f64>)> = series
        .iter()
        .map(|_| (vec![0.0; length], vec![0.0; length]))
        .collect();
    match options.offset {
        StackOffset::Zero | StackOffset::Normalize => {
            let mut above = vec![0.0f64; length];
            let mut below = vec![0.0f64; length];
            for &k in &order {
                for index in 0..length {
                    let value = value_at(k, index);
                    let (low, high) = if value < 0.0 {
                        let top = below[index];
                        below[index] += value;
                        (below[index], top)
                    } else {
                        let bottom = above[index];
                        above[index] += value;
                        (bottom, above[index])
                    };
                    bands[k].0[index] = low;
                    bands[k].1[index] = high;
                }
            }
            if options.offset == StackOffset::Normalize {
                for (low, high) in &mut bands {
                    for index in 0..length {
                        let total =
                            if high[index] > 0.0 || (low[index] == 0.0 && high[index] == 0.0) {
                                above[index]
                            } else {
                                -below[index]
                            };
                        if total > 0.0 {
                            low[index] /= total;
                            high[index] /= total;
                        }
                    }
                }
            }
        }
        StackOffset::Center => {
            let mut running = vec![0.0f64; length];
            let mut lowest = vec![0.0f64; length];
            let mut highest = vec![0.0f64; length];
            for &k in &order {
                for index in 0..length {
                    let value = value_at(k, index);
                    let from = running[index];
                    running[index] += value;
                    let to = running[index];
                    bands[k].0[index] = from.min(to);
                    bands[k].1[index] = from.max(to);
                    lowest[index] = lowest[index].min(to);
                    highest[index] = highest[index].max(to);
                }
            }
            for (low, high) in &mut bands {
                for index in 0..length {
                    let shift = (lowest[index] + highest[index]) / 2.0;
                    low[index] -= shift;
                    high[index] -= shift;
                }
            }
        }
    }
    bands
}

#[cfg(test)]
#[path = "tests/stack_tests.rs"]
mod tests;
