//! Rolling windows: reduces over a sliding window with the shared reducer
//! vocabulary.

use std::collections::VecDeque;

/// Where a window sits relative to the position it answers for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum WindowAnchor {
    /// The window starts at the position and looks ahead.
    Start,
    /// The window is centered on the position — the static-chart convention
    /// for smoothing, with no lag; an even size leans one value to the left.
    Middle,
    /// The window ends at the position — trailing, the streaming convention
    /// (the default).
    #[default]
    End,
}

/// A sliding window of `size` values, reduced at every position.
///
/// The window trails by default ([`WindowAnchor::End`]); [`Window::anchor`]
/// centers it or starts it at the position. Positions whose window runs past
/// an end reduce the partial window (no warm-up gap in the chart) unless
/// [`Window::strict`] asks for gaps there. Gaps (`NaN`) are excluded from each
/// window's reduction; empty finite windows follow the reducer's policy: zero
/// for count/sum and a gap otherwise. The named methods are sugar over
/// [`reduce`](Window::reduce) with the crate's one [`Reducer`](super::Reducer)
/// vocabulary, shared with [`super::Agg`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    size: usize,
    anchor: WindowAnchor,
    strict: bool,
}

impl Window {
    /// A window of `size` trailing values.
    ///
    /// # Panics
    ///
    /// Panics if `size` is zero.
    pub fn new(size: usize) -> Window {
        assert!(size > 0, "Window::new requires a non-zero size");
        Window {
            size,
            anchor: WindowAnchor::End,
            strict: false,
        }
    }

    /// Sets where the window sits relative to each position.
    #[must_use]
    pub fn anchor(mut self, anchor: WindowAnchor) -> Window {
        self.anchor = anchor;
        self
    }

    /// Reduces only complete windows: a position whose window would run past
    /// either end answers with a gap instead of a partial window — the
    /// reporting convention, where a mean of two values must not pose as a
    /// mean of twenty.
    #[must_use]
    pub fn strict(mut self) -> Window {
        self.strict = true;
        self
    }

    /// Applies any named [`Reducer`](super::Reducer) over each window —
    /// rolling medians, percentiles, and deviations included:
    /// `window.reduce(&latencies, Reducer::Percentile(0.95))`.
    pub fn reduce(&self, values: &[f64], reducer: super::Reducer) -> Vec<f64> {
        let n = values.len();
        let trailing = self.trailing(values, reducer);
        // How far the window's start sits before the position it answers for.
        let back = match self.anchor {
            WindowAnchor::Start => 0,
            WindowAnchor::Middle => (self.size - 1) / 2,
            WindowAnchor::End => self.size - 1,
        };
        if back == self.size - 1 && !self.strict {
            return trailing;
        }
        // A leading pass answers the windows that run past the end: the
        // trailing reduce of the reversed series, read back in order — with
        // first and last swapped, since the reversal swaps them.
        let leading: Option<Vec<f64>> = (back < self.size - 1).then(|| {
            let mirrored = match reducer {
                super::Reducer::First => super::Reducer::Last,
                super::Reducer::Last => super::Reducer::First,
                other => other,
            };
            let mut reversed: Vec<f64> = values.to_vec();
            reversed.reverse();
            let mut reduced = self.trailing(&reversed, mirrored);
            reduced.reverse();
            reduced
        });
        (0..n)
            .map(|position| {
                let start = position as i64 - back as i64;
                let end = start + self.size as i64 - 1;
                let partial = start < 0 || end >= n as i64;
                if partial && self.strict {
                    return f64::NAN;
                }
                if end < n as i64 {
                    // A window ending inside the series is a trailing window
                    // (its start clips to zero exactly like a trailing one).
                    trailing[end as usize]
                } else {
                    leading
                        .as_ref()
                        .map_or(trailing[n - 1], |leading| leading[start.max(0) as usize])
                }
            })
            .collect()
    }

    /// The trailing reduce at every position, partial windows at the start.
    fn trailing(&self, values: &[f64], reducer: super::Reducer) -> Vec<f64> {
        match reducer {
            super::Reducer::Count => self.rolling_count(values),
            super::Reducer::Sum => self.rolling_additive(values, false),
            super::Reducer::Mean => self.rolling_additive(values, true),
            super::Reducer::Min => self.rolling_extreme(values, true),
            super::Reducer::Max => self.rolling_extreme(values, false),
            super::Reducer::Median => self.rolling_quantile(values, 0.5),
            super::Reducer::Percentile(position) => {
                assert!(
                    (0.0..=1.0).contains(&position),
                    "Reducer::Percentile requires a position in [0, 1]"
                );
                self.rolling_quantile(values, position)
            }
            super::Reducer::Deviation
            | super::Reducer::Variance
            | super::Reducer::StdErr
            | super::Reducer::First
            | super::Reducer::Last => self.rolling_recompute(values, reducer),
        }
    }

    /// The rolling mean.
    pub fn mean(&self, values: &[f64]) -> Vec<f64> {
        self.reduce(values, super::Reducer::Mean)
    }

    /// The rolling sum (0 when nothing is finite).
    pub fn sum(&self, values: &[f64]) -> Vec<f64> {
        self.reduce(values, super::Reducer::Sum)
    }

    /// The rolling median.
    pub fn median(&self, values: &[f64]) -> Vec<f64> {
        self.reduce(values, super::Reducer::Median)
    }

    /// The rolling minimum.
    pub fn min(&self, values: &[f64]) -> Vec<f64> {
        self.reduce(values, super::Reducer::Min)
    }

    /// The rolling maximum.
    pub fn max(&self, values: &[f64]) -> Vec<f64> {
        self.reduce(values, super::Reducer::Max)
    }

    fn rolling_count(&self, values: &[f64]) -> Vec<f64> {
        let mut count = 0usize;
        let mut reduced = Vec::with_capacity(values.len());
        for (index, &value) in values.iter().enumerate() {
            if index >= self.size && values[index - self.size].is_finite() {
                count -= 1;
            }
            if value.is_finite() {
                count += 1;
            }
            reduced.push(count as f64);
        }
        reduced
    }

    /// Rolling sum/mean share the same add/remove state. If a finite window's sum
    /// overflows, recompute that exceptional window so an infinity does not poison
    /// every later result; means use their robust one-shot reducer in that case.
    fn rolling_additive(&self, values: &[f64], mean: bool) -> Vec<f64> {
        let mut count = 0usize;
        let mut sum = 0.0;
        let mut reduced = Vec::with_capacity(values.len());
        for (end, &value) in values.iter().enumerate() {
            if end >= self.size {
                let outgoing = values[end - self.size];
                if outgoing.is_finite() {
                    count -= 1;
                    sum -= outgoing;
                }
            }
            if value.is_finite() {
                count += 1;
                sum += value;
            }

            let start = (end + 1).saturating_sub(self.size);
            if !sum.is_finite() {
                sum = values[start..=end]
                    .iter()
                    .copied()
                    .filter(|value| value.is_finite())
                    .sum();
            }
            reduced.push(if !mean {
                sum
            } else if count == 0 {
                f64::NAN
            } else if sum.is_finite() {
                sum / count as f64
            } else {
                super::Reducer::Mean.reduce(&values[start..=end])
            });
        }
        reduced
    }

    /// A monotonic deque keeps only candidates that can become the window's
    /// extremum. Every finite value enters and leaves at most once.
    fn rolling_extreme(&self, values: &[f64], minimum: bool) -> Vec<f64> {
        let mut candidates = VecDeque::<usize>::with_capacity(self.size.min(values.len()));
        let mut reduced = Vec::with_capacity(values.len());
        for (index, &value) in values.iter().enumerate() {
            let start = (index + 1).saturating_sub(self.size);
            while candidates
                .front()
                .is_some_and(|&candidate| candidate < start)
            {
                candidates.pop_front();
            }
            if value.is_finite() {
                while candidates.back().is_some_and(|&candidate| {
                    if minimum {
                        values[candidate] >= value
                    } else {
                        values[candidate] <= value
                    }
                }) {
                    candidates.pop_back();
                }
                candidates.push_back(index);
            }
            reduced.push(
                candidates
                    .front()
                    .map_or(f64::NAN, |&candidate| values[candidate]),
            );
        }
        reduced
    }

    /// The reducers with no incremental form here reduce each window afresh
    /// through the shared state — spreads (Welford would drift when values
    /// leave the window), first, last.
    fn rolling_recompute(&self, values: &[f64], reducer: super::Reducer) -> Vec<f64> {
        let mut reduced = Vec::with_capacity(values.len());
        for end in 0..values.len() {
            let start = (end + 1).saturating_sub(self.size);
            reduced.push(reducer.reduce(&values[start..=end]));
        }
        reduced
    }

    /// Order statistics are inherently buffered here. Reusing one sample buffer
    /// makes that cost explicit and avoids one allocation per output position.
    fn rolling_quantile(&self, values: &[f64], position: f64) -> Vec<f64> {
        let mut sample = Vec::with_capacity(self.size.min(values.len()));
        let mut reduced = Vec::with_capacity(values.len());
        for end in 0..values.len() {
            let start = (end + 1).saturating_sub(self.size);
            sample.clear();
            sample.extend(
                values[start..=end]
                    .iter()
                    .copied()
                    .filter(|value| value.is_finite()),
            );
            if sample.is_empty() {
                reduced.push(f64::NAN);
            } else {
                sample.sort_by(f64::total_cmp);
                reduced.push(super::reducer::quantile_sorted(&sample, position));
            }
        }
        reduced
    }
}

#[cfg(test)]
#[path = "tests/window_tests.rs"]
mod tests;
