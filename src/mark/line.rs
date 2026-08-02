//! The line mark: a polyline through ordered points.

use std::ops::Range;
use std::sync::Arc;

use crate::data::{IntoSeries, Series};
use crate::render::Color;

/// A polyline through ordered points; gaps (`NaN`) break it visibly.
///
/// Data enters three ways: y values against their indices ([`Line::y`]), paired
/// series ([`Line::xy`]), or a function sampled at raster resolution
/// ([`Line::function`]).
#[derive(Clone)]
pub struct Line<'a> {
    pub(crate) source: Source<'a>,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
}

#[derive(Clone)]
pub(crate) enum Source<'a> {
    Points {
        x: Option<Series<'a>>,
        y: Series<'a>,
    },
    Function {
        domain: (f64, f64),
        function: Arc<dyn Fn(f64) -> f64 + Send + Sync>,
    },
}

impl<'a> Line<'a> {
    /// A line through `values` plotted against their indices `0, 1, 2, …`.
    pub fn y(values: impl IntoSeries<'a>) -> Line<'a> {
        Line {
            source: Source::Points {
                x: None,
                y: values.into_series(),
            },
            color: None,
            label: None,
        }
    }

    /// A line through the points `(x[i], y[i])`.
    ///
    /// # Panics
    ///
    /// Panics if the two series have different lengths.
    pub fn xy(x: impl IntoSeries<'a>, y: impl IntoSeries<'a>) -> Line<'a> {
        let x = x.into_series();
        let y = y.into_series();
        assert_eq!(x.len(), y.len(), "Line::xy requires series of equal length");
        Line {
            source: Source::Points { x: Some(x), y },
            color: None,
            label: None,
        }
    }

    /// A line through `function`, sampled once per subpixel column over `domain`.
    ///
    /// Sampling at raster resolution means the drawn curve is as smooth as the
    /// surface can express, regardless of the domain's size.
    ///
    /// # Panics
    ///
    /// Panics if the domain is not finite or is empty.
    pub fn function(
        domain: Range<f64>,
        function: impl Fn(f64) -> f64 + Send + Sync + 'static,
    ) -> Line<'a> {
        assert!(
            domain.start.is_finite() && domain.end.is_finite() && domain.start < domain.end,
            "Line::function requires a finite, non-empty domain"
        );
        Line {
            source: Source::Function {
                domain: (domain.start, domain.end),
                function: Arc::new(function),
            },
            color: None,
            label: None,
        }
    }

    /// Sets an explicit color; without one, layers take colors from the palette.
    #[must_use]
    pub fn color(mut self, color: Color) -> Line<'a> {
        self.color = Some(color);
        self
    }

    /// Names this layer in the legend. The legend appears once any layer is
    /// labeled (and the frame is tall enough for it).
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Line<'a> {
        self.label = Some(label.into());
        self
    }

    /// Detaches from any borrowed storage, making the mark `'static`.
    pub fn into_owned(self) -> Line<'static> {
        Line {
            source: match self.source {
                Source::Points { x, y } => Source::Points {
                    x: x.map(Series::into_owned),
                    y: y.into_owned(),
                },
                Source::Function { domain, function } => Source::Function { domain, function },
            },
            color: self.color,
            label: self.label,
        }
    }
}

impl std::fmt::Debug for Line<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Line");
        match &self.source {
            Source::Points { x, y } => {
                debug.field("points", &y.len());
                debug.field("indexed", &x.is_none());
            }
            Source::Function { domain, .. } => {
                debug.field("function_over", domain);
            }
        }
        debug.field("color", &self.color).finish()
    }
}

#[cfg(test)]
#[path = "tests/line_tests.rs"]
mod tests;
