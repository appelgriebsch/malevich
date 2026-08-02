//! The linear scale: an affine map from data domain to raster range.

/// A linear mapping from a data domain onto a raster range.
///
/// Both ends are inclusive; the range may run backwards (raster y grows downward
/// while data y grows upward, and this is where that flip lives). `NaN` maps to
/// `NaN` — gaps stay gaps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Linear {
    domain: (f64, f64),
    range: (f64, f64),
}

impl Linear {
    /// Creates the map `domain -> range`. A degenerate domain (both ends equal)
    /// maps every value to the middle of the range.
    pub fn new(domain: (f64, f64), range: (f64, f64)) -> Linear {
        Linear { domain, range }
    }

    /// Maps a data value into the range.
    pub fn map(&self, value: f64) -> f64 {
        let (d0, d1) = self.domain;
        let (r0, r1) = self.range;
        if d0 == d1 {
            return if value.is_nan() {
                value
            } else {
                (r0 + r1) / 2.0
            };
        }
        r0 + (value - d0) / (d1 - d0) * (r1 - r0)
    }
}
