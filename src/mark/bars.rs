//! The bars mark: values as filled columns over a categorical axis.

use crate::data::{IntoSeries, Series};
use crate::render::Color;

/// Filled vertical bars, one per category, rising (or falling) from a zero
/// baseline — the y domain always includes zero, because bar length *is* the
/// encoding.
///
/// Bars put a band scale on the x axis. Other layers in the same plot position
/// their x values against category indices: `0.0` is the center of the first band,
/// `1.0` the second, and so on.
#[derive(Clone)]
pub struct Bars<'a> {
    pub(crate) categories: Vec<String>,
    pub(crate) values: Series<'a>,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
}

impl<'a> Bars<'a> {
    /// Bars for `values`, one per category.
    ///
    /// # Panics
    ///
    /// Panics if the number of categories differs from the number of values.
    pub fn new(
        categories: impl IntoIterator<Item = impl Into<String>>,
        values: impl IntoSeries<'a>,
    ) -> Bars<'a> {
        let categories: Vec<String> = categories.into_iter().map(Into::into).collect();
        let values = values.into_series();
        assert_eq!(
            categories.len(),
            values.len(),
            "Bars::new requires one category per value"
        );
        Bars {
            categories,
            values,
            color: None,
            label: None,
        }
    }

    /// Sets an explicit color; without one, layers take colors from the palette.
    #[must_use]
    pub fn color(mut self, color: Color) -> Bars<'a> {
        self.color = Some(color);
        self
    }

    /// Names this layer in the legend. The legend appears once any layer is
    /// labeled (and the frame is tall enough for it).
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Bars<'a> {
        self.label = Some(label.into());
        self
    }

    /// Detaches from any borrowed storage, making the mark `'static`.
    pub fn into_owned(self) -> Bars<'static> {
        Bars {
            categories: self.categories,
            values: self.values.into_owned(),
            color: self.color,
            label: self.label,
        }
    }
}

impl std::fmt::Debug for Bars<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bars")
            .field("bars", &self.categories.len())
            .field("color", &self.color)
            .finish()
    }
}

#[cfg(test)]
#[path = "tests/bars_tests.rs"]
mod tests;
