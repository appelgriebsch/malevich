//! Group-by aggregation with the shared reducer vocabulary.

use crate::data::IntoSeries;

/// Values grouped by string keys, ready to reduce.
///
/// Groups keep first-appearance order; non-finite values are ignored. The reducer
/// names — `count`, `sum`, `mean`, `min`, `max`, `median` — are the crate's shared
/// vocabulary: the same names will reduce bins and windows. Each reducer returns
/// `(categories, values)`, which feeds [`crate::mark::Bars::new`] directly.
///
/// ```
/// use malevich::stat::Agg;
///
/// let (categories, means) = Agg::by(
///     ["a", "b", "a", "b"],
///     &[1.0, 10.0, 3.0, 30.0][..],
/// )
/// .mean();
/// assert_eq!(categories, ["a", "b"]);
/// assert_eq!(means, [2.0, 20.0]);
/// ```
#[derive(Debug, Clone)]
pub struct Agg {
    keys: Vec<String>,
    groups: Vec<Vec<f64>>,
}

impl Agg {
    /// Groups `values` by their paired `keys`.
    ///
    /// # Panics
    ///
    /// Panics if there are not exactly as many keys as values.
    pub fn by<'a>(
        keys: impl IntoIterator<Item = impl Into<String>>,
        values: impl IntoSeries<'a>,
    ) -> Agg {
        let keys: Vec<String> = keys.into_iter().map(Into::into).collect();
        let values = values.into_series();
        assert_eq!(
            keys.len(),
            values.len(),
            "Agg::by requires one key per value"
        );
        let mut result = Agg {
            keys: Vec::new(),
            groups: Vec::new(),
        };
        for (key, value) in keys.into_iter().zip(values.iter()) {
            let index = match result.keys.iter().position(|k| *k == key) {
                Some(index) => index,
                None => {
                    result.keys.push(key);
                    result.groups.push(Vec::new());
                    result.keys.len() - 1
                }
            };
            if value.is_finite() {
                result.groups[index].push(value);
            }
        }
        result
    }

    /// The number of finite values per group.
    pub fn count(self) -> (Vec<String>, Vec<f64>) {
        let counts = self.groups.iter().map(|g| g.len() as f64).collect();
        (self.keys, counts)
    }

    /// The sum per group (0 for empty groups).
    pub fn sum(self) -> (Vec<String>, Vec<f64>) {
        let sums = self.groups.iter().map(|g| g.iter().sum()).collect();
        (self.keys, sums)
    }

    /// The mean per group (a gap for empty groups).
    pub fn mean(self) -> (Vec<String>, Vec<f64>) {
        let means = self
            .groups
            .iter()
            .map(|g| {
                if g.is_empty() {
                    f64::NAN
                } else {
                    g.iter().sum::<f64>() / g.len() as f64
                }
            })
            .collect();
        (self.keys, means)
    }

    /// The minimum per group (a gap for empty groups).
    pub fn min(self) -> (Vec<String>, Vec<f64>) {
        let mins = self
            .groups
            .iter()
            .map(|g| g.iter().copied().fold(f64::INFINITY, f64::min))
            .map(|v| if v.is_finite() { v } else { f64::NAN })
            .collect();
        (self.keys, mins)
    }

    /// The maximum per group (a gap for empty groups).
    pub fn max(self) -> (Vec<String>, Vec<f64>) {
        let maxes = self
            .groups
            .iter()
            .map(|g| g.iter().copied().fold(f64::NEG_INFINITY, f64::max))
            .map(|v| if v.is_finite() { v } else { f64::NAN })
            .collect();
        (self.keys, maxes)
    }

    /// The median per group (a gap for empty groups; the mean of the middle pair
    /// for even counts).
    pub fn median(mut self) -> (Vec<String>, Vec<f64>) {
        let medians = self
            .groups
            .iter_mut()
            .map(|group| {
                if group.is_empty() {
                    return f64::NAN;
                }
                let middle = group.len() / 2;
                let (_, upper, _) = group.select_nth_unstable_by(middle, f64::total_cmp);
                let upper = *upper;
                if group.len() % 2 == 1 {
                    upper
                } else {
                    let lower = group[..middle]
                        .iter()
                        .copied()
                        .fold(f64::NEG_INFINITY, f64::max);
                    (lower + upper) / 2.0
                }
            })
            .collect();
        (self.keys, medians)
    }
}

#[cfg(test)]
#[path = "tests/agg_tests.rs"]
mod tests;
