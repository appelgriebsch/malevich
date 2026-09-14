//! Dodging: side-by-side positions for grouped bars.

/// Positions for grouped (dodged) bars: one position series per value series,
/// each series' indices `0, 1, 2, …` shifted by `(k - (n - 1) / 2) · step` for
/// series `k` of `n`, so the group centers on its band with adjacent positions
/// `step` apart. Feeds [`Bars::at`](crate::mark::Bars::at), one layer per
/// series, on a bands axis (`0.0` is the first band's center) — the sibling of
/// [`stack`](crate::stat::stack) for bars beside each other rather than on top
/// of each other. Give `Bars::at` the same `step` as its width and the bars
/// touch; give it less and a gap separates them, which keeps the series apart
/// in colorless output.
///
/// Shorter series get positions for their own length; `step` is not checked
/// here — `Bars::at` checks its own width.
///
/// ```
/// let positions = malevich::stat::dodge(&[&[1.0, 2.0], &[3.0, 4.0]], 0.5);
/// assert_eq!(positions[0], [-0.25, 0.75]);
/// assert_eq!(positions[1], [0.25, 1.25]);
/// ```
pub fn dodge(series: &[&[f64]], step: f64) -> Vec<Vec<f64>> {
    let count = series.len();
    series
        .iter()
        .enumerate()
        .map(|(k, values)| {
            let offset = (k as f64 - (count as f64 - 1.0) / 2.0) * step;
            (0..values.len()).map(|i| i as f64 + offset).collect()
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/dodge_tests.rs"]
mod tests;
