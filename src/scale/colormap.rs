//! Colormaps: continuous value-to-color scales for gridded marks.

use crate::render::Color;

/// A sequential colormap: linear interpolation through RGB stops.
///
/// The default approximates viridis — perceptually ordered, colorblind-safe,
/// readable on dark and light backgrounds. Any custom map is just a list of stops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colormap {
    stops: &'static [(u8, u8, u8)],
}

impl Colormap {
    /// The default sequential map (a viridis approximation).
    pub const DEFAULT: Colormap = Colormap {
        stops: &[
            (68, 1, 84),
            (59, 82, 139),
            (33, 145, 140),
            (94, 201, 98),
            (253, 231, 37),
        ],
    };

    /// A custom colormap over evenly spaced RGB stops.
    ///
    /// # Panics
    ///
    /// Panics with fewer than two stops.
    pub const fn new(stops: &'static [(u8, u8, u8)]) -> Colormap {
        assert!(
            stops.len() >= 2,
            "Colormap::new requires at least two stops"
        );
        Colormap { stops }
    }

    /// The color at `position` in `[0, 1]` (clamped; `NaN` maps to the low end).
    pub fn color(&self, position: f64) -> Color {
        let position = if position.is_finite() {
            position.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let scaled = position * (self.stops.len() - 1) as f64;
        let index = (scaled as usize).min(self.stops.len() - 2);
        let t = scaled - index as f64;
        let (r0, g0, b0) = self.stops[index];
        let (r1, g1, b1) = self.stops[index + 1];
        let lerp = |a: u8, b: u8| (f64::from(a) + (f64::from(b) - f64::from(a)) * t) as u8;
        Color::Rgb(lerp(r0, r1), lerp(g0, g1), lerp(b0, b1))
    }
}

impl Default for Colormap {
    fn default() -> Colormap {
        Colormap::DEFAULT
    }
}

#[cfg(test)]
#[path = "tests/colormap_tests.rs"]
mod tests;
