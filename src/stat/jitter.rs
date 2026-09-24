//! Jitter: quasirandom offsets that spread coincident points apart.

/// Positions spread by a van der Corput offset: point `i` moves by
/// `(v(i) − ½) · width`, where `v` is the base-2 van der Corput sequence, so
/// a strip of points fills its band evenly instead of clumping the way a
/// random draw would — and there is no seed to forget, so the plot stays a
/// value. Gaps stay gaps.
///
/// The strip beside a box plot, the "rain" of a raincloud: feed the band
/// indices in and hand the result to [`Points::xy`](crate::Points::xy).
///
/// ```
/// let spread = malevich::stat::jitter(&[0.0, 0.0, 0.0, 0.0], 2.0);
/// assert_eq!(spread, [0.0, -0.5, 0.5, -0.75]);
/// ```
pub fn jitter(positions: &[f64], width: f64) -> Vec<f64> {
    positions
        .iter()
        .enumerate()
        .map(|(index, &position)| {
            if position.is_finite() {
                position + (van_der_corput(index as u64 + 1) - 0.5) * width
            } else {
                f64::NAN
            }
        })
        .collect()
}

/// The base-2 van der Corput sequence at `index`: the binary digits of
/// `index` mirrored across the point.
fn van_der_corput(mut index: u64) -> f64 {
    let mut value = 0.0;
    let mut denominator = 2.0;
    while index > 0 {
        value += (index & 1) as f64 / denominator;
        index >>= 1;
        denominator *= 2.0;
    }
    value
}

#[cfg(test)]
#[path = "tests/jitter_tests.rs"]
mod tests;
