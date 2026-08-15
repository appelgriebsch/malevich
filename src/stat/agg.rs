//! Group-by aggregation with the shared reducer vocabulary.

use crate::data::IntoSeries;

/// Values grouped by string keys, ready to reduce.
///
/// Groups keep first-appearance order; non-finite values are ignored. The named
/// methods are sugar over [`reduce`](Agg::reduce) with the crate's one
/// [`Reducer`](super::Reducer) vocabulary — the same names reduce windows and
/// bins, and `reduce(Reducer::Percentile(q))` opens the quantiles. Each returns
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
    /// Applies any named [`Reducer`] per group — the percentile door:
    /// `agg.reduce(Reducer::Percentile(0.95))`.
    pub fn reduce(self, reducer: super::Reducer) -> (Vec<String>, Vec<f64>) {
        let values = self
            .groups
            .iter()
            .map(|group| reducer.reduce(group))
            .collect();
        (self.keys, values)
    }

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
        self.reduce(super::Reducer::Count)
    }

    /// The sum per group (0 for empty groups).
    pub fn sum(self) -> (Vec<String>, Vec<f64>) {
        self.reduce(super::Reducer::Sum)
    }

    /// The mean per group (a gap for empty groups).
    pub fn mean(self) -> (Vec<String>, Vec<f64>) {
        self.reduce(super::Reducer::Mean)
    }

    /// The minimum per group (a gap for empty groups).
    pub fn min(self) -> (Vec<String>, Vec<f64>) {
        self.reduce(super::Reducer::Min)
    }

    /// The maximum per group (a gap for empty groups).
    pub fn max(self) -> (Vec<String>, Vec<f64>) {
        self.reduce(super::Reducer::Max)
    }

    /// The median per group (a gap for empty groups; the mean of the middle pair
    /// for even counts).
    pub fn median(self) -> (Vec<String>, Vec<f64>) {
        self.reduce(super::Reducer::Median)
    }
}

#[cfg(test)]
#[path = "tests/agg_tests.rs"]
mod tests;
