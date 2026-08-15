//! Rolling windows: trailing reduces with the shared reducer vocabulary.

/// A trailing window of `size` values, reduced at every position.
///
/// The first positions reduce partial windows (no warm-up gap in the chart), gaps
/// (`NaN`) are excluded from each window's reduction, and a window with nothing
/// finite reduces to a gap. The named methods are sugar over
/// [`reduce`](Window::reduce) with the crate's one [`Reducer`](super::Reducer)
/// vocabulary, shared with [`super::Agg`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    size: usize,
}

impl Window {
    /// A window of `size` trailing values.
    ///
    /// # Panics
    ///
    /// Panics if `size` is zero.
    pub fn new(size: usize) -> Window {
        assert!(size > 0, "Window::new requires a non-zero size");
        Window { size }
    }

    /// Applies any named [`Reducer`](super::Reducer) over each trailing
    /// window — rolling medians and percentiles included:
    /// `window.reduce(&latencies, Reducer::Percentile(0.95))`.
    pub fn reduce(&self, values: &[f64], reducer: super::Reducer) -> Vec<f64> {
        self.rolling(values, |window| reducer.reduce(window))
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

    fn rolling(&self, values: &[f64], reducer: impl Fn(&[f64]) -> f64) -> Vec<f64> {
        (0..values.len())
            .map(|end| {
                let start = (end + 1).saturating_sub(self.size);
                reducer(&values[start..=end])
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "tests/window_tests.rs"]
mod tests;
