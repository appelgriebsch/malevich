//! Marks: the geometric primitives that draw data.
//!
//! A mark binds channels (data) to a drawing rule; it holds no scales, no layout, and
//! no terminal state. Marks are layered onto a [`crate::Plot`], which resolves shared
//! scales across all layers and rasterizes. [`Mark`] is the closed set of them —
//! chart types compose marks, they never extend the set.

mod area;
mod bars;
mod cells;
mod line;
mod points;
mod range;
mod rule;
mod text;

pub use area::Area;
pub use bars::Bars;
pub(crate) use bars::Placement;
pub use cells::Cells;
pub(crate) use line::Source;
pub use line::{Line, LineStyle};
pub use points::{PointStyle, Points};
pub use range::Range;
pub(crate) use range::RangePlacement;
pub(crate) use rule::Orientation;
pub use rule::Rule;
pub use text::Text;

/// Any mark, ready to be layered onto a plot.
///
/// Constructed via `From` — `plot.layer(Line::y(&data))` works directly.
#[non_exhaustive]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Mark<'a> {
    /// A polyline through ordered points.
    Line(Line<'a>),
    /// Unconnected point markers.
    Points(Points<'a>),
    /// Filled columns over a categorical axis.
    Bars(Bars<'a>),
    /// A filled region between two edges.
    Area(Area<'a>),
    /// A value grid drawn as shaded, colored cells.
    Cells(Cells<'a>),
    /// Vertical intervals: error bars, boxes, event ticks.
    Range(Range<'a>),
    /// A reference line across the plot.
    Rule(Rule),
    /// A text annotation at data coordinates.
    Text(Text),
}

impl<'a> Mark<'a> {
    /// Detaches from any borrowed storage, making the mark `'static`.
    pub fn into_owned(self) -> Mark<'static> {
        match self {
            Mark::Line(line) => Mark::Line(line.into_owned()),
            Mark::Points(points) => Mark::Points(points.into_owned()),
            Mark::Bars(bars) => Mark::Bars(bars.into_owned()),
            Mark::Area(area) => Mark::Area(area.into_owned()),
            Mark::Cells(cells) => Mark::Cells(cells.into_owned()),
            Mark::Range(range) => Mark::Range(range.into_owned()),
            Mark::Rule(rule) => Mark::Rule(rule),
            Mark::Text(text) => Mark::Text(text),
        }
    }

    /// Checks this mark's channel invariants, returning the first violation.
    ///
    /// The constructors enforce these already; this re-checks a mark that arrived
    /// another way (deserialization) so the fallible API can report bad specs.
    pub(crate) fn validate(&self) -> Result<(), crate::Error> {
        match self {
            Mark::Line(line) => {
                if let Source::Points { x: Some(x), y } = &line.source {
                    pair("Line: x and y", x.len(), y.len())?;
                }
            }
            Mark::Points(points) => {
                if let Some(x) = &points.x {
                    pair("Points: x and y", x.len(), points.y.len())?;
                }
            }
            Mark::Bars(bars) => {
                if let Placement::Bands(categories) = &bars.placement {
                    pair(
                        "Bars: categories and values",
                        categories.len(),
                        bars.values.len(),
                    )?;
                }
            }
            Mark::Area(area) => {
                if let Some(x) = &area.x {
                    pair("Area: x and high", x.len(), area.high.len())?;
                }
                if let Some(low) = &area.low {
                    pair("Area: low and high", low.len(), area.high.len())?;
                }
            }
            Mark::Cells(cells) => {
                if cells.columns == 0 {
                    return Err(crate::Error::EmptyDimension {
                        what: "Cells columns",
                    });
                }
                if !cells.values.len().is_multiple_of(cells.columns) {
                    return Err(crate::Error::NonRectangular {
                        mark: "Cells",
                        shape: (cells.values.len(), cells.columns),
                    });
                }
                if cells.colormap.stop_count() < 2 {
                    return Err(crate::Error::EmptyDimension {
                        what: "Colormap stops",
                    });
                }
                if let Some((x, y)) = cells.extents {
                    if !(x.0.is_finite() && x.1.is_finite() && y.0.is_finite() && y.1.is_finite()) {
                        return Err(crate::Error::InvalidParameter {
                            detail: "Cells extents must be finite",
                        });
                    }
                    if x.0 == x.1 || y.0 == y.1 {
                        return Err(crate::Error::InvalidParameter {
                            detail: "Cells extents must be non-empty",
                        });
                    }
                }
            }
            Mark::Range(range) => {
                let n = range.low.len();
                pair("Range: low and high", n, range.high.len())?;
                match &range.placement {
                    RangePlacement::Numeric(Some(x)) => pair("Range: x and low", x.len(), n)?,
                    RangePlacement::Bands(categories) => {
                        pair("Range: categories and low", categories.len(), n)?
                    }
                    RangePlacement::Numeric(None) => {}
                }
                if let Some((low, high)) = &range.body {
                    pair("Range: body low and high", low.len(), high.len())?;
                    pair("Range: body and low", low.len(), n)?;
                }
                if let Some(marker) = &range.marker {
                    pair("Range: marker and low", marker.len(), n)?;
                }
            }
            Mark::Rule(_) | Mark::Text(_) => {}
        }
        Ok(())
    }
}

/// Errors unless the two channel lengths match.
fn pair(mark: &'static str, a: usize, b: usize) -> Result<(), crate::Error> {
    if a == b {
        Ok(())
    } else {
        Err(crate::Error::UnequalChannels {
            mark,
            lengths: (a, b),
        })
    }
}

impl<'a> From<Line<'a>> for Mark<'a> {
    fn from(line: Line<'a>) -> Mark<'a> {
        Mark::Line(line)
    }
}

impl<'a> From<Points<'a>> for Mark<'a> {
    fn from(points: Points<'a>) -> Mark<'a> {
        Mark::Points(points)
    }
}

impl<'a> From<Bars<'a>> for Mark<'a> {
    fn from(bars: Bars<'a>) -> Mark<'a> {
        Mark::Bars(bars)
    }
}

impl<'a> From<Area<'a>> for Mark<'a> {
    fn from(area: Area<'a>) -> Mark<'a> {
        Mark::Area(area)
    }
}

impl<'a> From<Rule> for Mark<'a> {
    fn from(rule: Rule) -> Mark<'a> {
        Mark::Rule(rule)
    }
}

impl<'a> From<Text> for Mark<'a> {
    fn from(text: Text) -> Mark<'a> {
        Mark::Text(text)
    }
}

impl<'a> From<Cells<'a>> for Mark<'a> {
    fn from(cells: Cells<'a>) -> Mark<'a> {
        Mark::Cells(cells)
    }
}

impl<'a> From<Range<'a>> for Mark<'a> {
    fn from(range: Range<'a>) -> Mark<'a> {
        Mark::Range(range)
    }
}
