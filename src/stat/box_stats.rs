//! Box-plot statistics: the five-number summary plus outliers.

/// The five-number summary of a sample, Tukey style.
///
/// Quartiles use the type-7 estimator (the R default: linear interpolation of the
/// order statistics); whiskers extend to the most extreme values within 1.5 × IQR of
/// the quartiles; everything beyond is an outlier.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxStats {
    /// The first quartile.
    pub q1: f64,
    /// The median.
    pub median: f64,
    /// The third quartile.
    pub q3: f64,
    /// The lowest value within `q1 - 1.5 * IQR`.
    pub whisker_low: f64,
    /// The highest value within `q3 + 1.5 * IQR`.
    pub whisker_high: f64,
    /// Values beyond the whiskers, in order.
    pub outliers: Vec<f64>,
}

impl BoxStats {
    /// The summary of `values`, ignoring non-finite entries. `None` when nothing
    /// finite remains.
    pub fn of(values: &[f64]) -> Option<BoxStats> {
        let mut sorted: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
        if sorted.is_empty() {
            return None;
        }
        sorted.sort_by(f64::total_cmp);
        let q1 = quantile_sorted(&sorted, 0.25);
        let median = quantile_sorted(&sorted, 0.5);
        let q3 = quantile_sorted(&sorted, 0.75);
        let reach = 1.5 * (q3 - q1);
        let (fence_low, fence_high) = (q1 - reach, q3 + reach);
        let whisker_low = sorted
            .iter()
            .copied()
            .find(|&v| v >= fence_low)
            .unwrap_or(q1);
        let whisker_high = sorted
            .iter()
            .rev()
            .copied()
            .find(|&v| v <= fence_high)
            .unwrap_or(q3);
        let outliers = sorted
            .iter()
            .copied()
            .filter(|&v| v < fence_low || v > fence_high)
            .collect();
        Some(BoxStats {
            q1,
            median,
            q3,
            whisker_low,
            whisker_high,
            outliers,
        })
    }
}

/// The type-7 quantile of an ascending-sorted, non-empty slice.
fn quantile_sorted(sorted: &[f64], p: f64) -> f64 {
    let position = (sorted.len() - 1) as f64 * p;
    let index = position.floor() as usize;
    let fraction = position - index as f64;
    if index + 1 < sorted.len() {
        sorted[index] + fraction * (sorted[index + 1] - sorted[index])
    } else {
        sorted[index]
    }
}

#[cfg(test)]
#[path = "tests/box_stats_tests.rs"]
mod tests;
