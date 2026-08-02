//! Histogram binning: uniform bins with nice-number edges.

use crate::scale::Ticks;

/// A uniform histogram: `bins` counting buckets of `width`, starting at `start`.
///
/// A mergeable monoid: partial histograms over chunks combine with [`Bins::merge`].
/// Values outside the covered range are ignored (the [`Bins::auto`] constructor
/// sizes the range to the data, so nothing drops there); the right edge of the last
/// bin is inclusive, so the maximum lands inside.
#[derive(Debug, Clone, PartialEq)]
pub struct Bins {
    start: f64,
    width: f64,
    counts: Vec<u64>,
}

impl Bins {
    /// An empty histogram of `bins` buckets of `width`, starting at `start`.
    ///
    /// # Panics
    ///
    /// Panics if `width` is not finite and positive, or `bins` is zero.
    pub fn new(start: f64, width: f64, bins: usize) -> Bins {
        assert!(
            start.is_finite() && width.is_finite() && width > 0.0 && bins > 0,
            "Bins::new requires a finite start, positive width, and at least one bin"
        );
        Bins {
            start,
            width,
            counts: vec![0; bins],
        }
    }

    /// Bins sized to the data: bin count by the larger of Sturges' rule and
    /// Freedman–Diaconis (the NumPy `auto` policy), capped at `limit`, with widths
    /// and edges snapped to the same nice decimals ticks use. `None` without finite
    /// values.
    pub fn auto(values: &[f64], limit: usize) -> Option<Bins> {
        let mut finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
        if finite.is_empty() {
            return None;
        }
        let n = finite.len();
        let (min, max) = finite
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
                (lo.min(v), hi.max(v))
            });
        if min == max {
            let mut bins = Bins::new(min - 0.5, 1.0, 1);
            bins.counts[0] = n as u64;
            return Some(bins);
        }

        let sturges = (n as f64).log2().ceil() as usize + 1;
        let quarter = n / 4;
        let (_, q1, _) = finite.select_nth_unstable_by(quarter, f64::total_cmp);
        let q1 = *q1;
        let upper = (3 * n) / 4;
        let (_, q3, _) = finite.select_nth_unstable_by(upper.min(n - 1), f64::total_cmp);
        let q3 = *q3;
        let iqr = q3 - q1;
        let fd = if iqr > 0.0 {
            let width = 2.0 * iqr / (n as f64).cbrt();
            ((max - min) / width).ceil() as usize
        } else {
            0
        };
        let target = sturges.max(fd).clamp(1, limit.max(1));

        // Snap the bin width and edges to the nice decimals the tick engine picks,
        // so bin boundaries land on readable numbers.
        let ticks = Ticks::linear(min, max, target.min(50));
        let width = if ticks.step() > 0.0 {
            ticks.step()
        } else {
            (max - min) / target as f64
        };
        let start = (min / width).floor() * width;
        let bins = (((max - start) / width).ceil() as usize).max(1);
        let mut result = Bins::new(start, width, bins.min(limit.max(1) * 2));
        for &value in &finite {
            result.add(value);
        }
        Some(result)
    }

    /// Counts one value; non-finite and out-of-range values are ignored.
    pub fn add(&mut self, value: f64) {
        if !value.is_finite() {
            return;
        }
        let position = (value - self.start) / self.width;
        if position < 0.0 {
            return;
        }
        let index = position as usize;
        if index < self.counts.len() {
            self.counts[index] += 1;
        } else if index == self.counts.len() && value <= self.end() {
            // The last bin's right edge is inclusive.
            *self.counts.last_mut().expect("at least one bin") += 1;
        }
    }

    /// Merges another histogram with the same geometry into this one.
    ///
    /// # Panics
    ///
    /// Panics if the two histograms have different starts, widths, or bin counts.
    pub fn merge(&mut self, other: &Bins) {
        assert!(
            self.start == other.start
                && self.width == other.width
                && self.counts.len() == other.counts.len(),
            "Bins::merge requires identical geometry"
        );
        for (mine, theirs) in self.counts.iter_mut().zip(other.counts.iter()) {
            *mine += theirs;
        }
    }

    /// The left edge of the first bin.
    pub fn start(&self) -> f64 {
        self.start
    }

    /// The width of every bin.
    pub fn width(&self) -> f64 {
        self.width
    }

    /// The right edge of the last bin.
    pub fn end(&self) -> f64 {
        self.start + self.width * self.counts.len() as f64
    }

    /// The per-bin counts, in order.
    pub fn counts(&self) -> &[u64] {
        &self.counts
    }
}

#[cfg(test)]
#[path = "tests/bin_tests.rs"]
mod tests;
