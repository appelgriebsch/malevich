//! `Plot`: the retained chart description, and its resolve → layout → rasterize
//! pipeline.

use super::frame::Frame;
use crate::mark::Mark;
use crate::render::Surface;

/// A retained chart description: layers of marks plus furniture.
///
/// A plot is a plain value — build it anywhere, clone it, send it across threads,
/// render it many times. Rendering is a pure function of the plot and a [`Frame`]:
/// no global state, no terminal access, no panics (undersized frames shed furniture
/// instead of failing).
///
/// ```
/// use malevich::{Frame, Line, Plot};
///
/// let plot = Plot::new()
///     .layer(Line::xy(&[0.0, 1.0, 2.0][..], &[1.0, 3.0, 2.0][..]))
///     .title("example");
/// let text = plot.render(&Frame::plain(40, 10));
/// assert!(text.contains("example"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Plot<'a> {
    layers: Vec<Mark<'a>>,
    title: Option<String>,
    log_x: bool,
    log_y: bool,
    time_x: bool,
}

impl<'a> Plot<'a> {
    /// An empty plot with no layers and no furniture.
    pub fn new() -> Plot<'a> {
        Plot {
            layers: Vec::new(),
            title: None,
            log_x: false,
            log_y: false,
            time_x: false,
        }
    }

    /// Reads the x axis as time: unix seconds (UTC), with calendar-aligned ticks
    /// and multi-scale labels (`14:05`, `Aug 2`, `2027`). Takes precedence over
    /// [`Plot::log_x`]; ignored when a bars layer owns the x axis.
    #[must_use]
    pub fn time_x(mut self) -> Plot<'a> {
        self.time_x = true;
        self
    }

    /// Puts the x axis on a base-10 logarithmic scale: decade ticks (`10²`-style),
    /// and values at or below zero become gaps — a log axis cannot place them
    /// honestly. Ignored when a bars layer owns the x axis.
    #[must_use]
    pub fn log_x(mut self) -> Plot<'a> {
        self.log_x = true;
        self
    }

    /// Puts the y axis on a base-10 logarithmic scale: decade ticks (`10²`-style),
    /// and values at or below zero become gaps — a log axis cannot place them
    /// honestly.
    #[must_use]
    pub fn log_y(mut self) -> Plot<'a> {
        self.log_y = true;
        self
    }

    /// Adds a mark as the next layer. Layers share scales: domains are the union of
    /// all layers' data, resolved at render time. A [`crate::mark::Bars`] layer puts
    /// a band scale on the x axis; other layers then position x against category
    /// indices (0 is the first band's center).
    #[must_use]
    pub fn layer(mut self, mark: impl Into<Mark<'a>>) -> Plot<'a> {
        self.layers.push(mark.into());
        self
    }

    /// Sets the title, shown centered above the plot (shed first when space runs out).
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Plot<'a> {
        self.title = Some(title.into());
        self
    }

    /// Detaches from any borrowed storage, making the plot `'static`.
    pub fn into_owned(self) -> Plot<'static> {
        Plot {
            layers: self.layers.into_iter().map(Mark::into_owned).collect(),
            title: self.title,
            log_x: self.log_x,
            log_y: self.log_y,
            time_x: self.time_x,
        }
    }

    /// Renders into a string according to the frame's charset and color mode.
    pub fn render(&self, frame: &Frame) -> String {
        self.rasterize(frame).encode(frame.color)
    }

    fn rasterize(&self, frame: &Frame) -> Surface {
        let mut surface = Surface::new(frame.width, frame.height, frame.charset);
        if frame.width == 0 || frame.height == 0 {
            return surface;
        }
        let (px, _) = frame.charset.pixels_per_cell();
        let layers = super::resolve::resolve(&self.layers, frame.width * px, &frame.theme.palette);
        let layout = super::layout::Layout::compute(
            frame,
            &layers,
            self.title.is_some(),
            (self.time_x, self.log_x, self.log_y),
        );
        super::chrome::draw(&mut surface, &layout, self.title.as_deref(), &layers);
        super::draw::layers(&mut surface, &layout, &layers);
        surface
    }
}

impl std::fmt::Display for Plot<'_> {
    /// Renders with [`Frame::detect`]: the one-line `println!("{plot}")` path.
    /// Detection assumes stdout; for full control use [`Plot::render`].
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render(&Frame::detect()))
    }
}

#[cfg(test)]
#[path = "tests/plot_tests.rs"]
mod tests;
