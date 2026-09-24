//! Box-plot statistics: the five-number summary plus outliers.

/// How far a box plot's whiskers reach.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Whiskers {
    /// To the most extreme values within `k × IQR` of the quartiles — Tukey's
    /// rule, `1.5` the convention and the default; everything beyond is an
    /// outlier.
    Tukey(f64),
    /// To the type-7 quantiles at two positions in `[0, 1]`, `Percentiles(0.05,
    /// 0.95)` say; values outside are outliers.
    Percentiles(f64, f64),
    /// To the minimum and maximum: nothing is an outlier.
    MinMax,
}

impl Default for Whiskers {
    fn default() -> Whiskers {
        Whiskers::Tukey(1.5)
    }
}

impl Whiskers {
    /// Checks the rule's parameters.
    pub(crate) fn validate(self) -> crate::Result<()> {
        match self {
            Whiskers::Tukey(reach) if !(reach.is_finite() && reach >= 0.0) => {
                Err(crate::Error::InvalidParameter {
                    detail: "a Tukey whisker reach must be finite and non-negative",
                })
            }
            Whiskers::Percentiles(low, high)
                if !((0.0..=1.0).contains(&low) && (0.0..=1.0).contains(&high) && low <= high) =>
            {
                Err(crate::Error::InvalidParameter {
                    detail: "whisker percentiles must be ascending positions in [0, 1]",
                })
            }
            _ => Ok(()),
        }
    }
}

/// The five-number summary of a sample, Tukey style.
///
/// Quartiles use the type-7 estimator (the R default: linear interpolation of the
/// order statistics); whiskers extend to the most extreme values within 1.5 × IQR of
/// the quartiles by default — [`BoxStats::of_with`] chooses another
/// [`Whiskers`] rule; everything beyond is an outlier.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxStats {
    /// The first quartile.
    pub q1: f64,
    /// The median.
    pub median: f64,
    /// The third quartile.
    pub q3: f64,
    /// The lowest value within the whisker rule's reach.
    pub whisker_low: f64,
    /// The highest value within the whisker rule's reach.
    pub whisker_high: f64,
    /// Values beyond the whiskers, in order.
    pub outliers: Vec<f64>,
}

impl BoxStats {
    /// The summary of `values`, ignoring non-finite entries, with Tukey's
    /// 1.5 × IQR whiskers. `None` when nothing finite remains.
    pub fn of(values: &[f64]) -> Option<BoxStats> {
        BoxStats::of_with(values, Whiskers::default())
    }

    /// The summary with a chosen whisker rule. The quartiles are the same
    /// type-7 estimates whatever the rule.
    ///
    /// # Panics
    ///
    /// Panics if the rule's parameters are invalid: a negative or non-finite
    /// Tukey reach, or percentiles outside `[0, 1]` or descending.
    pub fn of_with(values: &[f64], whiskers: Whiskers) -> Option<BoxStats> {
        whiskers
            .validate()
            .expect("BoxStats::of_with requires a valid whisker rule");
        let mut sorted: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
        if sorted.is_empty() {
            return None;
        }
        sorted.sort_by(f64::total_cmp);
        let quantile = |p: f64| super::reducer::quantile_sorted(&sorted, p);
        let q1 = quantile(0.25);
        let median = quantile(0.5);
        let q3 = quantile(0.75);
        let (fence_low, fence_high) = match whiskers {
            Whiskers::Tukey(reach) => {
                let reach = reach * (q3 - q1);
                (q1 - reach, q3 + reach)
            }
            Whiskers::Percentiles(low, high) => (quantile(low), quantile(high)),
            Whiskers::MinMax => (sorted[0], sorted[sorted.len() - 1]),
        };
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

#[cfg(test)]
#[path = "tests/box_stats_tests.rs"]
mod tests;
