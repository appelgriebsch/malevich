//! `Plot`: the retained chart description, and its resolve → layout → rasterize
//! pipeline.

use super::frame::Frame;
use crate::mark::Mark;
use crate::render::Surface;
use crate::scale::Scale;

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
    x: Scale,
    y: Scale,
    x_label: Option<String>,
    y_label: Option<String>,
    x_domain: Option<(f64, f64)>,
    y_domain: Option<(f64, f64)>,
}

impl<'a> Plot<'a> {
    /// An empty plot with no layers and no furniture.
    pub fn new() -> Plot<'a> {
        Plot {
            layers: Vec::new(),
            title: None,
            x: Scale::Linear,
            y: Scale::Linear,
            x_label: None,
            y_label: None,
            x_domain: None,
            y_domain: None,
        }
    }

    /// Fixes the x axis to `[min, max]` instead of fitting the data — matplotlib's
    /// `xlim`. Data outside clips honestly. Ignored on a bands axis.
    ///
    /// # Panics
    ///
    /// Panics if the bounds are not finite.
    #[must_use]
    pub fn x_domain(mut self, min: f64, max: f64) -> Plot<'a> {
        assert!(
            min.is_finite() && max.is_finite(),
            "Plot::x_domain requires finite bounds"
        );
        self.x_domain = Some((min.min(max), max.max(min)));
        self
    }

    /// Fixes the y axis to `[min, max]` instead of fitting the data — matplotlib's
    /// `ylim`. Data outside clips honestly.
    ///
    /// # Panics
    ///
    /// Panics if the bounds are not finite.
    #[must_use]
    pub fn y_domain(mut self, min: f64, max: f64) -> Plot<'a> {
        assert!(
            min.is_finite() && max.is_finite(),
            "Plot::y_domain requires finite bounds"
        );
        self.y_domain = Some((min.min(max), max.max(min)));
        self
    }

    /// Sets the x axis scale. Band layers (bars, band-placed ranges) imply
    /// [`Scale::Bands`] when none is set explicitly.
    #[must_use]
    pub fn x_scale(mut self, scale: Scale) -> Plot<'a> {
        self.x = scale;
        self
    }

    /// Sets the y axis scale.
    ///
    /// # Panics
    ///
    /// Panics on [`Scale::Bands`] — categorical y axes are not supported yet.
    #[must_use]
    pub fn y_scale(mut self, scale: Scale) -> Plot<'a> {
        assert!(
            !matches!(scale, Scale::Bands(_)),
            "categorical y axes are not supported yet"
        );
        self.y = scale;
        self
    }

    /// Sugar for [`Plot::x_scale`] with [`Scale::Time`]: unix seconds (UTC) with
    /// calendar-aligned, multi-scale tick labels (`14:05`, `Aug 2`, `2027`).
    #[must_use]
    pub fn time_x(self) -> Plot<'a> {
        self.x_scale(Scale::Time)
    }

    /// Sugar for [`Plot::x_scale`] with [`Scale::Log`]: decade ticks, and values at
    /// or below zero become gaps — a log axis cannot place them honestly.
    #[must_use]
    pub fn log_x(self) -> Plot<'a> {
        self.x_scale(Scale::Log)
    }

    /// Sugar for [`Plot::y_scale`] with [`Scale::Log`]: decade ticks, and values at
    /// or below zero become gaps — a log axis cannot place them honestly.
    #[must_use]
    pub fn log_y(self) -> Plot<'a> {
        self.y_scale(Scale::Log)
    }

    /// Titles the x axis, centered under its tick labels.
    #[must_use]
    pub fn x_label(mut self, label: impl Into<String>) -> Plot<'a> {
        self.x_label = Some(label.into());
        self
    }

    /// Titles the y axis, written vertically along the left edge.
    #[must_use]
    pub fn y_label(mut self, label: impl Into<String>) -> Plot<'a> {
        self.y_label = Some(label.into());
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
            x: self.x,
            y: self.y,
            x_label: self.x_label,
            y_label: self.y_label,
            x_domain: self.x_domain,
            y_domain: self.y_domain,
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
            (&self.x, &self.y),
            (self.x_label.as_deref(), self.y_label.as_deref()),
            (self.x_domain, self.y_domain),
        );
        super::chrome::draw(
            &mut surface,
            &layout,
            self.title.as_deref(),
            (self.x_label.as_deref(), self.y_label.as_deref()),
            &layers,
        );
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
