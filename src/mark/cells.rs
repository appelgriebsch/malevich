//! The cells mark: a value grid drawn as shaded, colored cells.

use crate::data::{IntoSeries, Series};
use crate::scale::Colormap;

/// A grid of values — a heatmap, a matrix, a 2D histogram.
///
/// Values normalize to the grid's own finite extent and render as a shade ramp
/// (`░▒▓█`) colored by a [`Colormap`]: the value is carried by glyph *and* color, so
/// the grid stays readable even in plain, colorless output. Gaps (`NaN`) render as
/// blanks. Row 0 is the bottom row — matrix y grows upward like any other y axis.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cells<'a> {
    pub(crate) columns: usize,
    pub(crate) values: Series<'a>,
    pub(crate) extents: Option<((f64, f64), (f64, f64))>,
    pub(crate) colormap: Colormap,
}

impl<'a> Cells<'a> {
    /// A grid from row-major `values`, `columns` wide; the row count is
    /// `values.len() / columns`. Axes show cell indices unless
    /// [`Cells::extents`] maps them to data coordinates.
    ///
    /// # Panics
    ///
    /// Panics if `columns` is zero or does not divide the value count evenly.
    pub fn matrix(columns: usize, values: impl IntoSeries<'a>) -> Cells<'a> {
        let values = values.into_series();
        assert!(
            columns > 0 && values.len() % columns == 0,
            "Cells::matrix requires columns to divide the value count evenly"
        );
        Cells {
            columns,
            values,
            extents: None,
            colormap: Colormap::DEFAULT,
        }
    }

    /// Maps the grid onto data coordinates: the x axis spans `x`, the y axis `y`.
    ///
    /// # Panics
    ///
    /// Panics if the extents are not finite or either span is empty. Reversed
    /// endpoints are accepted and flip that grid axis.
    #[must_use]
    pub fn extents(mut self, x: (f64, f64), y: (f64, f64)) -> Cells<'a> {
        assert!(
            x.0.is_finite()
                && x.1.is_finite()
                && y.0.is_finite()
                && y.1.is_finite()
                && x.0 != x.1
                && y.0 != y.1,
            "Cells::extents requires finite, non-empty bounds"
        );
        self.extents = Some((x, y));
        self
    }

    /// Sets the colormap; the default approximates viridis.
    #[must_use]
    pub fn colormap(mut self, colormap: Colormap) -> Cells<'a> {
        self.colormap = colormap;
        self
    }

    /// Detaches from any borrowed storage, making the mark `'static`.
    pub fn into_owned(self) -> Cells<'static> {
        Cells {
            columns: self.columns,
            values: self.values.into_owned(),
            extents: self.extents,
            colormap: self.colormap,
        }
    }
}

impl std::fmt::Debug for Cells<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cells")
            .field("columns", &self.columns)
            .field("rows", &(self.values.len() / self.columns.max(1)))
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[path = "tests/cells_tests.rs"]
mod tests;
