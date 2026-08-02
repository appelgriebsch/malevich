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
mod rule;
mod text;

pub use area::Area;
pub use bars::Bars;
pub(crate) use bars::Placement;
pub use cells::Cells;
pub use line::Line;
pub(crate) use line::Source;
pub use points::Points;
pub(crate) use rule::Orientation;
pub use rule::Rule;
pub use text::Text;

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
    /// Filled columns over a categorical axis.
    Bars(Bars<'a>),
    /// A filled region between two edges.
    Area(Area<'a>),
    /// A value grid drawn as shaded, colored cells.
    Cells(Cells<'a>),
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
            Mark::Rule(rule) => Mark::Rule(rule),
            Mark::Text(text) => Mark::Text(text),
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
