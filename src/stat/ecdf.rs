//! The empirical cumulative distribution function.

/// The ECDF of `values`: sorted finite values paired with the fraction of the data
/// at or below each. Feeds a step line — see [`crate::ecdf`].
///
/// Returns empty vectors when nothing is finite.
pub fn ecdf(values: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let mut sorted: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    sorted.sort_by(f64::total_cmp);
    let n = sorted.len() as f64;
    let fractions = (1..=sorted.len()).map(|rank| rank as f64 / n).collect();
    (sorted, fractions)
}

#[cfg(test)]
#[path = "tests/ecdf_tests.rs"]
mod tests;
