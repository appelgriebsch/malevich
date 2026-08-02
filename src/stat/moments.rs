//! Streaming moments: count, mean, variance, and extrema in one pass.

/// Running statistics over a stream of values: Welford's online algorithm for the
/// mean and variance, plus the extrema.
///
/// A mergeable monoid: partial results over chunks combine with [`Moments::merge`]
/// (Chan's parallel formula) into the same statistics a single pass produces.
/// Non-finite values are ignored.
#[derive(Debug, Clone, Copy, Default)]
pub struct Moments {
    count: u64,
    mean: f64,
    m2: f64,
    min: f64,
    max: f64,
}

impl Moments {
    /// An empty accumulator.
    pub fn new() -> Moments {
        Moments {
            count: 0,
            mean: 0.0,
            m2: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    /// Accumulates one value; non-finite values are ignored.
    pub fn add(&mut self, value: f64) {
        if !value.is_finite() {
            return;
        }
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        self.m2 += delta * (value - self.mean);
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    /// Merges another accumulator into this one.
    pub fn merge(&mut self, other: &Moments) {
        if other.count == 0 {
            return;
        }
        if self.count == 0 {
            *self = *other;
            return;
        }
        let total = self.count + other.count;
        let delta = other.mean - self.mean;
        self.mean += delta * other.count as f64 / total as f64;
        self.m2 += other.m2 + delta * delta * self.count as f64 * other.count as f64 / total as f64;
        self.count = total;
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    /// The number of finite values seen.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// The mean, or `None` before any value.
    pub fn mean(&self) -> Option<f64> {
        (self.count > 0).then_some(self.mean)
    }

    /// The population variance, or `None` before any value.
    pub fn variance(&self) -> Option<f64> {
        (self.count > 0).then(|| self.m2 / self.count as f64)
    }

    /// The population standard deviation, or `None` before any value.
    pub fn standard_deviation(&self) -> Option<f64> {
        self.variance().map(f64::sqrt)
    }

    /// The smallest value, or `None` before any value.
    pub fn min(&self) -> Option<f64> {
        (self.count > 0).then_some(self.min)
    }

    /// The largest value, or `None` before any value.
    pub fn max(&self) -> Option<f64> {
        (self.count > 0).then_some(self.max)
    }
}

#[cfg(test)]
#[path = "tests/moments_tests.rs"]
mod tests;
