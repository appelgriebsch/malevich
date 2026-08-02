//! Largest-Triangle-Three-Buckets downsampling.
//!
//! Steinarsson, "Downsampling Time Series for Visual Representation" (MSc thesis,
//! University of Iceland, 2013). Shape-preserving, count-targeted downsampling:
//! pick, per bucket, the point forming the largest triangle with its neighbors.
//! Complementary to [`super::m4`] — LTTB targets a point *count* and preserves
//! visual character, M4 targets a raster *width* and is pixel-exact there.

/// Downsamples the series to at most `target` points, keeping the first and last.
///
/// Expects x ascending (as [`super::m4`] does); pairs with a non-finite coordinate
/// are dropped before sampling. Returns the input (filtered) when it is already at
/// or under `target`, and empty vectors when nothing finite remains. A `target`
/// below 3 is treated as 3 — LTTB needs both endpoints plus room to choose.
pub fn lttb(x: &[f64], y: &[f64], target: usize) -> (Vec<f64>, Vec<f64>) {
    let target = target.max(3);
    let points: Vec<(f64, f64)> = x
        .iter()
        .zip(y.iter())
        .filter(|(x, y)| x.is_finite() && y.is_finite())
        .map(|(&x, &y)| (x, y))
        .collect();
    if points.len() <= target {
        return points.into_iter().unzip();
    }

    let mut kept: Vec<(f64, f64)> = Vec::with_capacity(target);
    kept.push(points[0]);
    // Interior buckets partition everything between the two endpoints.
    let interior = target - 2;
    let span = (points.len() - 2) as f64 / interior as f64;
    let mut previous = points[0];
    for bucket in 0..interior {
        let start = 1 + (bucket as f64 * span) as usize;
        let end = 1 + (((bucket + 1) as f64 * span) as usize).min(points.len() - 2);
        // The anchor ahead: the average of the next bucket (or the last point).
        let ahead_start = end;
        let ahead_end = if bucket + 1 == interior {
            points.len()
        } else {
            1 + (((bucket + 2) as f64 * span) as usize).min(points.len() - 1)
        };
        let ahead_count = (ahead_end - ahead_start).max(1) as f64;
        let ahead = points[ahead_start..ahead_end.max(ahead_start + 1)]
            .iter()
            .fold((0.0, 0.0), |acc, point| (acc.0 + point.0, acc.1 + point.1));
        let ahead = (ahead.0 / ahead_count, ahead.1 / ahead_count);

        let mut best = points[start];
        let mut best_area = -1.0;
        for &point in &points[start..end.max(start + 1)] {
            let area = ((previous.0 - ahead.0) * (point.1 - previous.1)
                - (previous.0 - point.0) * (ahead.1 - previous.1))
                .abs();
            if area > best_area {
                best_area = area;
                best = point;
            }
        }
        kept.push(best);
        previous = best;
    }
    kept.push(points[points.len() - 1]);
    kept.into_iter().unzip()
}

#[cfg(test)]
#[path = "tests/lttb_tests.rs"]
mod tests;
