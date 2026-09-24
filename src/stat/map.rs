//! Series maps: length-preserving transforms of one ordered series — running
//! sums, differences, ranks, and normalization to a basis.

use super::Reducer;

/// The running sum of `values`. A gap (`NaN`) stays a gap in the output and
/// leaves the sum untouched, so the total resumes after it rather than
/// absorbing an invented zero.
///
/// ```
/// assert_eq!(malevich::stat::cumsum(&[1.0, 2.0, 3.0]), [1.0, 3.0, 6.0]);
/// ```
pub fn cumsum(values: &[f64]) -> Vec<f64> {
    let mut total = 0.0f64;
    values
        .iter()
        .map(|&value| {
            if value.is_finite() {
                total += value;
                total
            } else {
                f64::NAN
            }
        })
        .collect()
}

/// The difference of each value from the one before it — the first position,
/// and any position beside a gap, is a gap. The day-over-day delta.
///
/// ```
/// let deltas = malevich::stat::diff(&[10.0, 12.0, 11.0]);
/// assert!(deltas[0].is_nan());
/// assert_eq!(&deltas[1..], [2.0, -1.0]);
/// ```
pub fn diff(values: &[f64]) -> Vec<f64> {
    values
        .iter()
        .enumerate()
        .map(
            |(index, &value)| match index.checked_sub(1).map(|i| values[i]) {
                Some(previous) if previous.is_finite() && value.is_finite() => value - previous,
                _ => f64::NAN,
            },
        )
        .collect()
}

/// The ascending rank of each finite value among the finite values, from `0`;
/// tied values share the lowest rank they span (the d3 convention). Gaps stay
/// gaps.
///
/// ```
/// assert_eq!(malevich::stat::rank(&[30.0, 10.0, 20.0, 10.0]), [3.0, 0.0, 2.0, 0.0]);
/// ```
pub fn rank(values: &[f64]) -> Vec<f64> {
    let mut order: Vec<usize> = (0..values.len())
        .filter(|&index| values[index].is_finite())
        .collect();
    order.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
    let mut ranks = vec![f64::NAN; values.len()];
    let mut rank = 0usize;
    for (position, &index) in order.iter().enumerate() {
        if position > 0 && values[order[position - 1]] < values[index] {
            rank = position;
        }
        ranks[index] = rank as f64;
    }
    ranks
}

/// Every value divided by one reduction of the whole series — `basis` in the
/// shared [`Reducer`] vocabulary: `First` for an index chart (times 100 for
/// index-to-100), `Max` for percent-of-peak, `Sum` for shares, `Mean` for
/// ratios to the mean. A basis of zero, or one with no answer, makes every
/// output a gap rather than an infinity.
///
/// ```
/// use malevich::stat::{Reducer, normalize};
///
/// assert_eq!(normalize(&[2.0, 4.0, 1.0], Reducer::Max), [0.5, 1.0, 0.25]);
/// assert_eq!(normalize(&[50.0, 60.0], Reducer::First), [1.0, 1.2]);
/// ```
pub fn normalize(values: &[f64], basis: Reducer) -> Vec<f64> {
    let divisor = basis.reduce(values);
    if !divisor.is_finite() || divisor == 0.0 {
        return vec![f64::NAN; values.len()];
    }
    values.iter().map(|&value| value / divisor).collect()
}

#[cfg(test)]
#[path = "tests/map_tests.rs"]
mod tests;
