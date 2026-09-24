//! Colormaps: continuous value-to-color scales for gridded marks.

use std::borrow::Cow;

use crate::render::Color;

/// A value compared by bit pattern, so the spec types that hold a colormap
/// keep the `Eq` they promise. Construction rejects non-finite values; a
/// deserialized non-finite one is caught by spec validation.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
struct Exact(f64);

impl PartialEq for Exact {
    fn eq(&self, other: &Exact) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for Exact {}

/// A continuous colormap: linear interpolation through RGB stops, optionally
/// centered on a data midpoint.
///
/// The named constants are a curated set that stays distinguishable down the
/// whole color ladder (truecolor → 256 → 16 → plain shade): sequential
/// [`VIRIDIS`](Colormap::VIRIDIS) (the default), [`MAGMA`](Colormap::MAGMA),
/// [`CIVIDIS`](Colormap::CIVIDIS), and [`GREYS`](Colormap::GREYS); diverging
/// [`RED_BLUE`](Colormap::RED_BLUE) and
/// [`PURPLE_ORANGE`](Colormap::PURPLE_ORANGE), whose ends are named in
/// low-to-high order. Any custom map is just a list of stops.
///
/// Diverging maps encode signed or centered data honestly only when anchored:
/// [`centered_at`](Colormap::centered_at) pins a data value (0 for
/// correlations, 1 for ratios) to the map's middle and spans the larger side
/// symmetrically, so equal magnitudes on either side get equal intensity.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Colormap {
    stops: Cow<'static, [(u8, u8, u8)]>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    midpoint: Option<Exact>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "core::ops::Not::not")
    )]
    log: bool,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    domain: Option<(Exact, Exact)>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    under: Option<Color>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    over: Option<Color>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    steps: Option<usize>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    thresholds: Option<Vec<Exact>>,
}

impl Colormap {
    /// The default sequential map: [`VIRIDIS`](Colormap::VIRIDIS).
    pub const DEFAULT: Colormap = Colormap::VIRIDIS;

    /// A viridis approximation — perceptually ordered, colorblind-safe,
    /// readable on dark and light backgrounds.
    pub const VIRIDIS: Colormap = Colormap::new(&[
        (68, 1, 84),
        (59, 82, 139),
        (33, 145, 140),
        (94, 201, 98),
        (253, 231, 37),
    ]);

    /// A magma approximation — perceptually ordered, near-black to pale yellow.
    pub const MAGMA: Colormap = Colormap::new(&[
        (0, 0, 4),
        (81, 18, 124),
        (183, 55, 121),
        (252, 137, 97),
        (252, 253, 191),
    ]);

    /// A cividis approximation — perceptually ordered, optimized for
    /// red-green color vision deficiency.
    pub const CIVIDIS: Colormap = Colormap::new(&[
        (0, 32, 77),
        (65, 77, 107),
        (124, 123, 120),
        (188, 175, 111),
        (255, 233, 69),
    ]);

    /// A plain grey ramp, dim to bright.
    pub const GREYS: Colormap = Colormap::new(&[(64, 64, 64), (250, 250, 250)]);

    /// Diverging red → neutral → blue (ColorBrewer RdBu). Anchor it with
    /// [`centered_at`](Colormap::centered_at).
    pub const RED_BLUE: Colormap = Colormap::new(&[
        (202, 0, 32),
        (244, 165, 130),
        (247, 247, 247),
        (146, 197, 222),
        (5, 113, 176),
    ]);

    /// Diverging purple → neutral → orange (ColorBrewer PuOr, low end purple).
    /// Anchor it with [`centered_at`](Colormap::centered_at).
    pub const PURPLE_ORANGE: Colormap = Colormap::new(&[
        (94, 60, 153),
        (178, 171, 210),
        (247, 247, 247),
        (253, 184, 99),
        (230, 97, 1),
    ]);

    /// The canonical names [`named`](Colormap::named) resolves, for help text
    /// and option listings.
    pub const NAMES: [&'static str; 6] = [
        "viridis",
        "magma",
        "cividis",
        "greys",
        "red-blue",
        "purple-orange",
    ];

    /// Looks up a named built-in map (see [`NAMES`](Colormap::NAMES);
    /// `"grays"` is accepted for `"greys"`). Diverging maps come back
    /// unanchored — apply [`centered_at`](Colormap::centered_at) to center
    /// them on a data value.
    pub fn named(name: &str) -> Option<Colormap> {
        match name {
            "viridis" => Some(Colormap::VIRIDIS),
            "magma" => Some(Colormap::MAGMA),
            "cividis" => Some(Colormap::CIVIDIS),
            "greys" | "grays" => Some(Colormap::GREYS),
            "red-blue" => Some(Colormap::RED_BLUE),
            "purple-orange" => Some(Colormap::PURPLE_ORANGE),
            _ => None,
        }
    }

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
        Colormap {
            stops: Cow::Borrowed(stops),
            midpoint: None,
            log: false,
            domain: None,
            under: None,
            over: None,
            steps: None,
            thresholds: None,
        }
    }

    /// Builds a colormap from runtime-owned, evenly spaced RGB stops.
    ///
    /// The vector is retained without copying, so generated palettes and palettes
    /// loaded from configuration do not need to be leaked into `'static` storage.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyDimension`](crate::Error::EmptyDimension) when fewer
    /// than two stops are supplied.
    pub fn try_from_stops(stops: Vec<(u8, u8, u8)>) -> crate::Result<Colormap> {
        if stops.len() < 2 {
            return Err(crate::Error::EmptyDimension {
                what: "Colormap stops",
            });
        }
        Ok(Colormap {
            stops: Cow::Owned(stops),
            midpoint: None,
            log: false,
            domain: None,
            under: None,
            over: None,
            steps: None,
            thresholds: None,
        })
    }

    /// Centers the map on a data value: `midpoint` maps to ramp position 0.5,
    /// and the value range spans the larger side symmetrically, so equal
    /// magnitudes on either side of the midpoint get equal intensity. The
    /// colorbar shows the symmetric range.
    ///
    /// # Panics
    ///
    /// Panics when `midpoint` is not finite.
    #[must_use]
    pub fn centered_at(mut self, midpoint: f64) -> Colormap {
        assert!(
            midpoint.is_finite(),
            "Colormap::centered_at requires a finite midpoint"
        );
        self.midpoint = Some(Exact(midpoint));
        self
    }

    /// Makes the ramp logarithmic: equal color steps for equal factors, so
    /// values spanning decades — attention weights, gradient magnitudes,
    /// spectral power — stay distinguishable instead of collapsing into the
    /// low end of a linear ramp. Values at or below zero have no logarithmic
    /// position and render as gaps, the same rule log axes follow. The
    /// colorbar shows decade ticks.
    ///
    /// Logarithmic and centered are mutually exclusive; combining them fails
    /// validation.
    #[must_use]
    pub fn log(mut self) -> Colormap {
        self.log = true;
        self
    }

    /// Whether the ramp is logarithmic.
    pub fn is_log(&self) -> bool {
        self.log
    }

    /// Fixes the value range the ramp spans, instead of the data's own
    /// extent — so two grids read on one scale, and a live grid keeps its
    /// colors as its values move. Values outside clamp to the ends unless
    /// [`under`](Colormap::under) / [`over`](Colormap::over) disclose them.
    /// The colorbar shows the fixed range; a centered map symmetrizes it
    /// around the midpoint; a log map needs a positive one.
    ///
    /// # Panics
    ///
    /// Panics when either bound is not finite, or `low` is not below `high`.
    #[must_use]
    pub fn domain(mut self, low: f64, high: f64) -> Colormap {
        assert!(
            low.is_finite() && high.is_finite() && low < high,
            "Colormap::domain requires finite, ascending bounds"
        );
        self.domain = Some((Exact(low), Exact(high)));
        self
    }

    /// The color for values below the range — matplotlib's `set_under`:
    /// out-of-range disclosed, not clamped into the ramp's lowest color.
    #[must_use]
    pub fn under(mut self, color: Color) -> Colormap {
        self.under = Some(color);
        self
    }

    /// The color for values above the range; see [`under`](Colormap::under).
    #[must_use]
    pub fn over(mut self, color: Color) -> Colormap {
        self.over = Some(color);
        self
    }

    /// Quantizes the ramp into `count` equal bands, each one color — the
    /// stepped map that makes a heatmap's levels countable. The colorbar
    /// draws the bands and labels their boundaries. Replaces any
    /// [`thresholds`](Colormap::thresholds).
    ///
    /// # Panics
    ///
    /// Panics when `count` is zero.
    #[must_use]
    pub fn steps(mut self, count: usize) -> Colormap {
        assert!(count >= 1, "Colormap::steps requires at least one band");
        self.steps = Some(count);
        self.thresholds = None;
        self
    }

    /// Splits the ramp at `thresholds` — `k` values, `k + 1` bands, each one
    /// color, the colorbar labeling the thresholds. Sorted and deduplicated;
    /// [`contourf`](crate::contourf) is a heatmap under a map split at the
    /// contour levels. Replaces any [`steps`](Colormap::steps).
    ///
    /// # Panics
    ///
    /// Panics when a threshold is not finite.
    #[must_use]
    pub fn thresholds(mut self, thresholds: impl IntoIterator<Item = f64>) -> Colormap {
        let mut values: Vec<f64> = thresholds.into_iter().collect();
        assert!(
            values.iter().all(|value| value.is_finite()),
            "Colormap::thresholds requires finite values"
        );
        values.sort_by(f64::total_cmp);
        values.dedup();
        self.thresholds = Some(values.into_iter().map(Exact).collect());
        self.steps = None;
        self
    }

    /// The fixed value range, when this map has one.
    pub fn fixed_domain(&self) -> Option<(f64, f64)> {
        self.domain.map(|(low, high)| (low.0, high.0))
    }

    /// The color disclosed for values below the range, when set.
    pub fn under_color(&self) -> Option<Color> {
        self.under
    }

    /// The color disclosed for values above the range, when set.
    pub fn over_color(&self) -> Option<Color> {
        self.over
    }

    /// The number of bands of a stepped map — `steps`, or one more than the
    /// thresholds — and `None` for a continuous ramp.
    pub fn bands(&self) -> Option<usize> {
        match (self.steps, &self.thresholds) {
            (Some(count), _) => Some(count),
            (None, Some(thresholds)) => Some(thresholds.len() + 1),
            (None, None) => None,
        }
    }

    /// The band boundaries in value space for data observed in `[low, high]`,
    /// ends included: the equal divisions of a stepped map (by decade on a
    /// log ramp), or the thresholds inside the range. Empty for a
    /// continuous ramp.
    pub fn boundaries(&self, low: f64, high: f64) -> Vec<f64> {
        let (start, end) = self.range(low, high);
        match (self.steps, &self.thresholds) {
            (Some(count), _) => (0..=count)
                .map(|band| {
                    let t = band as f64 / count as f64;
                    if self.log && start > 0.0 && end > 0.0 {
                        10f64.powf(crate::numeric::lerp(start.log10(), end.log10(), t))
                    } else {
                        crate::numeric::lerp(start, end, t)
                    }
                })
                .collect(),
            (None, Some(thresholds)) => std::iter::once(start)
                .chain(
                    thresholds
                        .iter()
                        .map(|threshold| threshold.0)
                        .filter(|threshold| *threshold > start && *threshold < end),
                )
                .chain(std::iter::once(end))
                .collect(),
            (None, None) => Vec::new(),
        }
    }

    /// The centered data value, when this map has one.
    pub fn midpoint(&self) -> Option<f64> {
        self.midpoint.map(|midpoint| midpoint.0)
    }

    /// The evenly spaced RGB stops, from the low end to the high end.
    pub fn stops(&self) -> &[(u8, u8, u8)] {
        &self.stops
    }

    /// Checks invariants after any construction path.
    pub(crate) fn validate(&self) -> crate::Result<()> {
        if self.stops.len() < 2 {
            return Err(crate::Error::EmptyDimension {
                what: "Colormap stops",
            });
        }
        if self
            .midpoint
            .is_some_and(|midpoint| !midpoint.0.is_finite())
        {
            return Err(crate::Error::InvalidParameter {
                detail: "a colormap midpoint must be finite",
            });
        }
        if self.log && self.midpoint.is_some() {
            return Err(crate::Error::InvalidParameter {
                detail: "a colormap cannot be centered and logarithmic at once",
            });
        }
        if let Some((low, high)) = self.fixed_domain() {
            if !(low.is_finite() && high.is_finite() && low < high) {
                return Err(crate::Error::InvalidParameter {
                    detail: "a colormap domain must be finite and ascending",
                });
            }
            if self.log && low <= 0.0 {
                return Err(crate::Error::IncompatibleScale {
                    detail: "a logarithmic colormap needs a positive domain",
                });
            }
        }
        if self.steps == Some(0) {
            return Err(crate::Error::InvalidParameter {
                detail: "a stepped colormap needs at least one band",
            });
        }
        if self.steps.is_some() && self.thresholds.is_some() {
            return Err(crate::Error::InvalidParameter {
                detail: "a colormap is stepped or thresholded, not both",
            });
        }
        if let Some(thresholds) = &self.thresholds {
            if thresholds.iter().any(|threshold| !threshold.0.is_finite()) {
                return Err(crate::Error::InvalidParameter {
                    detail: "colormap thresholds must be finite",
                });
            }
            if thresholds.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
                return Err(crate::Error::InvalidParameter {
                    detail: "colormap thresholds must ascend",
                });
            }
        }
        Ok(())
    }

    /// The value range the ramp is positioned over: the fixed domain when
    /// there is one, else the observed `[low, high]` — then, for a linear
    /// map, symmetrized around a midpoint.
    fn range(&self, low: f64, high: f64) -> (f64, f64) {
        let (low, high) = self.fixed_domain().unwrap_or((low, high));
        if self.log {
            (low, high)
        } else {
            self.display_domain(low, high)
        }
    }

    /// The band-center position of `position` on a stepped or thresholded
    /// map, or `position` itself on a continuous ramp. Thresholds are placed
    /// on the ramp over the range data was observed in.
    pub(crate) fn quantize(&self, position: f64, low: f64, high: f64) -> f64 {
        let position = if position.is_finite() {
            position.clamp(0.0, 1.0)
        } else {
            return position;
        };
        match (self.steps, &self.thresholds) {
            (Some(count), _) => {
                let band = ((position * count as f64).floor() as usize).min(count - 1);
                (band as f64 + 0.5) / count as f64
            }
            (None, Some(thresholds)) => {
                let (start, end) = self.range(low, high);
                let place = |value: f64| {
                    if self.log && start > 0.0 && end > 0.0 {
                        crate::numeric::inverse_lerp(start.log10(), end.log10(), value.log10())
                    } else {
                        crate::numeric::inverse_lerp(start, end, value)
                    }
                };
                let band = thresholds
                    .iter()
                    .filter(|threshold| place(threshold.0) <= position)
                    .count();
                (band as f64 + 0.5) / (thresholds.len() + 1) as f64
            }
            (None, None) => position,
        }
    }

    /// The ramp position and color for `value` among data observed in
    /// `[low, high]`, everything this map knows applied at once: the fixed
    /// domain, the midpoint or the decades, the bands of a stepped map, and
    /// the [`under`](Colormap::under) / [`over`](Colormap::over) colors for
    /// values outside the range (at position 0 or 1). `None` is a gap: a
    /// non-finite value, or one a log ramp cannot place.
    pub fn sample(&self, value: f64, low: f64, high: f64) -> Option<(f64, Color)> {
        if !value.is_finite() {
            return None;
        }
        let (start, end) = self.range(low, high);
        if value < start
            && let Some(under) = self.under
        {
            return Some((0.0, under));
        }
        if value > end
            && let Some(over) = self.over
        {
            return Some((1.0, over));
        }
        let position = self.position_in(value, low, high);
        if !position.is_finite() {
            return None;
        }
        let position = self.quantize(position, low, high);
        Some((position, self.color(position)))
    }

    /// The centered midpoint when it is usable; a non-finite one (possible only
    /// through deserialization) degrades to a linear map.
    fn active_midpoint(&self) -> Option<f64> {
        self.midpoint().filter(|midpoint| midpoint.is_finite())
    }

    /// The value range the ramp displays for data observed in `[low, high]`:
    /// the fixed domain when there is one, else the range itself for a
    /// linear map, symmetrized around the midpoint for a centered one.
    pub(crate) fn display_domain(&self, low: f64, high: f64) -> (f64, f64) {
        let (low, high) = self.fixed_domain().unwrap_or((low, high));
        match self.active_midpoint() {
            Some(midpoint) => {
                let half = (high - midpoint).max(midpoint - low);
                let half = if half > 0.0 { half } else { 1.0 };
                (midpoint - half, midpoint + half)
            }
            None => (low, high),
        }
    }

    /// The ramp position in `[0, 1]` for `value` among data observed in
    /// `[low, high]` — linear across the range, or centered per
    /// [`centered_at`](Colormap::centered_at). `NaN` maps to the low end,
    /// matching [`color`](Colormap::color).
    ///
    /// A [`log`](Colormap::log) ramp positions by decade instead, and returns
    /// `NaN` — a gap — for any value at or below zero, and for every value
    /// when the observed range is not positive.
    pub fn position_in(&self, value: f64, low: f64, high: f64) -> f64 {
        let (low, high) = self.fixed_domain().unwrap_or((low, high));
        if self.log {
            if !(value > 0.0 && low > 0.0 && high > 0.0) {
                return f64::NAN;
            }
            let (start, end) = (low.log10(), high.log10());
            let position = if end > start {
                crate::numeric::inverse_lerp(start, end, value.log10())
            } else {
                0.0
            };
            return if position.is_finite() {
                position.clamp(0.0, 1.0)
            } else {
                0.0
            };
        }
        let (start, end) = self.display_domain(low, high);
        let position = if end > start {
            crate::numeric::inverse_lerp(start, end, value)
        } else {
            0.0
        };
        if position.is_finite() {
            position.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// The color at `position` in `[0, 1]` (clamped; `NaN` maps to the low end).
    ///
    /// A colormap built through [`Colormap::new`] always has at least two stops;
    /// one deserialized with too few degrades gracefully rather than panicking.
    pub fn color(&self, position: f64) -> Color {
        match self.stops.len() {
            0 => return Color::Default,
            1 => {
                let (r, g, b) = self.stops[0];
                return Color::Rgb(r, g, b);
            }
            _ => {}
        }
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
