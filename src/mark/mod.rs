//! Marks: the geometric primitives that draw data.
//!
//! A mark binds channels (data) to a drawing rule; it holds no scales, no layout, and
//! no terminal state. Marks are layered onto a [`crate::Plot`], which resolves shared
//! scales across all layers and rasterizes. [`Mark`] is the closed set of them —
//! chart types compose marks, they never extend the set.

mod line;
mod points;

pub use line::Line;
pub(crate) use line::Source;
pub use points::Points;

/// Any mark, ready to be layered onto a plot.
///
/// Constructed via `From` — `plot.layer(Line::y(&data))` works directly.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Mark<'a> {
    /// A polyline through ordered points.
    Line(Line<'a>),
    /// Unconnected dots.
    Points(Points<'a>),
}

impl<'a> Mark<'a> {
    /// Detaches from any borrowed storage, making the mark `'static`.
    pub fn into_owned(self) -> Mark<'static> {
        match self {
            Mark::Line(line) => Mark::Line(line.into_owned()),
            Mark::Points(points) => Mark::Points(points.into_owned()),
        }
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
