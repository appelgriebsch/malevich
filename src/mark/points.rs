//! The points mark: unconnected dots at data positions.

use crate::data::{IntoSeries, Series};
use crate::render::Color;

/// Unconnected dots at data positions; gaps (`NaN`) simply have no dot.
#[derive(Clone)]
pub struct Points<'a> {
    pub(crate) x: Option<Series<'a>>,
    pub(crate) y: Series<'a>,
    pub(crate) color: Option<Color>,
}

impl<'a> Points<'a> {
    /// Dots for `values` plotted against their indices `0, 1, 2, …`.
    pub fn y(values: impl IntoSeries<'a>) -> Points<'a> {
        Points {
            x: None,
            y: values.into_series(),
            color: None,
        }
    }

    /// Dots at the positions `(x[i], y[i])`.
    ///
    /// # Panics
    ///
    /// Panics if the two series have different lengths.
    pub fn xy(x: impl IntoSeries<'a>, y: impl IntoSeries<'a>) -> Points<'a> {
        let x = x.into_series();
        let y = y.into_series();
        assert_eq!(
            x.len(),
            y.len(),
            "Points::xy requires series of equal length"
        );
        Points {
            x: Some(x),
            y,
            color: None,
        }
    }

    /// Sets an explicit color; without one, layers take colors from the palette.
    #[must_use]
    pub fn color(mut self, color: Color) -> Points<'a> {
        self.color = Some(color);
        self
    }

    /// Detaches from any borrowed storage, making the mark `'static`.
    pub fn into_owned(self) -> Points<'static> {
        Points {
            x: self.x.map(Series::into_owned),
            y: self.y.into_owned(),
            color: self.color,
        }
    }
}

impl std::fmt::Debug for Points<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Points")
            .field("points", &self.y.len())
            .field("indexed", &self.x.is_none())
            .field("color", &self.color)
            .finish()
    }
}

#[cfg(test)]
#[path = "tests/points_tests.rs"]
mod tests;
