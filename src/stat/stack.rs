//! Stacking: series into cumulative bands.

/// Stacks series into `(low, high)` bands: each series sits on the sum of the ones
/// before it. Feeds [`crate::mark::Area::between`] layers directly.
///
/// Gaps (`NaN`) contribute zero to the running sum — a missing slice of a stack has
/// no thickness; shorter series are treated as zero-padded.
pub fn stack(series: &[&[f64]]) -> Vec<(Vec<f64>, Vec<f64>)> {
    let length = series.iter().map(|s| s.len()).max().unwrap_or(0);
    let mut base = vec![0.0f64; length];
    series
        .iter()
        .map(|values| {
            let low = base.clone();
            for (index, floor) in base.iter_mut().enumerate() {
                let value = values.get(index).copied().unwrap_or(0.0);
                if value.is_finite() {
                    *floor += value;
                }
            }
            (low, base.clone())
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/stack_tests.rs"]
mod tests;
