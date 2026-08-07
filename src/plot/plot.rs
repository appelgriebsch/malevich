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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Plot<'a> {
    layers: Vec<Mark<'a>>,
    title: Option<String>,
    x: Scale,
    y: Scale,
    x_label: Option<String>,
    y_label: Option<String>,
    x_domain: Option<(f64, f64)>,
    y_domain: Option<(f64, f64)>,
    #[cfg_attr(feature = "serde", serde(default))]
    colorbar: bool,
}

impl<'a> Plot<'a> {
    /// An empty plot with no layers and no furniture.
    pub fn new() -> Plot<'a> {
        Plot {
            layers: Vec::new(),
            title: None,
            x: Scale::Auto,
            y: Scale::Auto,
            x_label: None,
            y_label: None,
            x_domain: None,
            y_domain: None,
            colorbar: false,
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

    /// Sets the x axis scale. Under [`Scale::Auto`] (the default) a bars or
    /// band-range layer makes the axis categorical; any scale set here is honored
    /// as-is, so an explicit choice is never overridden by a categorical layer.
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

    /// Shows a colorbar: a vertical strip of the colormap down the right edge,
    /// labeled with the value range it encodes. Applies to the plot's first
    /// [`Cells`](crate::Cells) layer (heatmaps, 2D histograms); ignored when there is
    /// none, or when the frame is too narrow to spare the room.
    #[must_use]
    pub fn colorbar(mut self) -> Plot<'a> {
        self.colorbar = true;
        self
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
            colorbar: self.colorbar,
        }
    }

    /// Renders into a string according to the frame's charset and color mode.
    pub fn render(&self, frame: &Frame) -> String {
        self.try_render_unvalidated(frame).unwrap_or_default()
    }

    /// Renders the complete plot as a self-contained HTML terminal card.
    ///
    /// The cell grid is placed in a styled `<pre>` element: default-colored
    /// chrome inherits the card foreground, while mark colors become concrete RGB
    /// spans. The frame's color mode is ignored because HTML always carries RGB;
    /// its size, charset, and theme still apply. Rendering is pure and deterministic
    /// for a given plot and frame.
    #[cfg(feature = "evcxr")]
    pub fn to_html(&self, frame: &Frame) -> String {
        use std::fmt::Write as _;

        let content = self.rasterize(frame).encode_html();
        let (background, foreground) = crate::render::card(frame.theme);
        let mut html = String::with_capacity(content.len() + 320);
        let _ = write!(
            html,
            "<pre style=\"margin:0;padding:12px 16px;border:0;border-radius:8px;box-sizing:border-box;display:inline-block;max-width:100%;overflow-x:auto;white-space:pre;font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:13px;line-height:1.1;background-color:{background};color:{foreground}\">{content}</pre>"
        );
        html
    }

    /// Displays this plot when it is the last expression in an Evcxr cell.
    ///
    /// Emits two representations and lets the frontend pick the richest it can
    /// draw: an HTML card (100×26 braille, dark theme) for Jupyter, and a terminal
    /// plot (80×24) for the terminal REPL, which cannot render HTML and would
    /// otherwise show nothing. With the `pixel` feature also enabled, the terminal
    /// block becomes a real sixel/kitty/iTerm2 image in a graphics-capable terminal
    /// (detected by sniffing the environment; evcxr pipes stdout, so no tty probe
    /// runs) and stays cells everywhere else. Use [`Plot::to_html`] with a custom
    /// [`Frame`] for explicit size, charset, or theme control.
    #[cfg(feature = "evcxr")]
    pub fn evcxr_display(&self) {
        let html = self.to_html(&Frame::plain(100, 26));
        let terminal = Frame::plain(80, 24);
        // `render_best` upgrades the terminal block to a real image when a graphics
        // protocol is sniffed, and is exactly `render` otherwise — so a pipe, an
        // unknown terminal, or a missing `pixel` feature all yield honest cells.
        #[cfg(feature = "pixel")]
        let plain = self.render_best(&terminal);
        #[cfg(not(feature = "pixel"))]
        let plain = self.render(&terminal);
        println!(
            "{}",
            crate::render::mime_bundle(&[("text/html", &html), ("text/plain", &plain)])
        );
    }

    /// Checks the spec against the invariants the constructors enforce — paired
    /// channel lengths, rectangular grids, valid colormaps — plus finite manual
    /// domains and scale/domain compatibility, without rendering. Returns the first
    /// problem as an [`Error`](crate::Error).
    ///
    /// [`Plot::render`] never fails: it sheds whatever it cannot draw. `validate` is
    /// the strict counterpart for a spec that arrived by deserialization or
    /// configuration, where you want a typed error rather than a quietly dropped
    /// mark. [`Plot::try_render`] does both in one call.
    ///
    /// ```
    /// let plot = malevich::line(&[1.0, 2.0, 3.0][..]);
    /// assert!(plot.validate().is_ok());
    /// ```
    pub fn validate(&self) -> crate::Result<()> {
        for layer in &self.layers {
            layer.validate()?;
        }
        // Categorical layers must agree on one ordered set of bands, and a numeric x
        // scale cannot host them — `Auto` adapts, but an explicit numeric choice is a
        // conflict, not an override.
        let mut bands: Option<&[String]> = match &self.x {
            Scale::Bands(bands) => Some(bands.as_slice()),
            _ => None,
        };
        for layer in &self.layers {
            let layer_bands = match layer {
                Mark::Bars(bars) => match &bars.placement {
                    crate::mark::Placement::Bands(bands) => Some(bands.as_slice()),
                    _ => None,
                },
                Mark::Range(range) => match &range.placement {
                    crate::mark::RangePlacement::Bands(bands) => Some(bands.as_slice()),
                    _ => None,
                },
                _ => None,
            };
            let Some(layer_bands) = layer_bands else {
                continue;
            };
            if matches!(self.x, Scale::Linear | Scale::Log | Scale::Time) {
                return Err(crate::Error::IncompatibleScale {
                    detail: "a categorical layer needs an Auto or Bands x scale",
                });
            }
            match bands {
                Some(existing) if existing != layer_bands => {
                    return Err(crate::Error::IncompatibleScale {
                        detail: "categorical layers disagree on their bands",
                    });
                }
                _ => bands = Some(layer_bands),
            }
        }
        for (axis, domain) in [("x", self.x_domain), ("y", self.y_domain)] {
            if let Some((lo, hi)) = domain
                && !(lo.is_finite() && hi.is_finite())
            {
                return Err(crate::Error::NonFiniteDomain { axis });
            }
        }
        if matches!(self.x, Scale::Log)
            && let Some((lo, _)) = self.x_domain
            && lo <= 0.0
        {
            return Err(crate::Error::IncompatibleScale {
                detail: "a log x axis needs a positive domain",
            });
        }
        if matches!(self.y, Scale::Log)
            && let Some((lo, _)) = self.y_domain
            && lo <= 0.0
        {
            return Err(crate::Error::IncompatibleScale {
                detail: "a log y axis needs a positive domain",
            });
        }
        Ok(())
    }

    /// [`Plot::validate`] followed by fallible rasterization and encoding: a
    /// rendered string, or the first spec, geometry, or allocation error.
    pub fn try_render(&self, frame: &Frame) -> crate::Result<String> {
        self.validate()?;
        self.try_render_unvalidated(frame)
    }

    pub(crate) fn try_render_unvalidated(&self, frame: &Frame) -> crate::Result<String> {
        self.try_rasterize(frame)?.try_encode(frame.color)
    }

    /// Renders into `frame` at the best graphics tier the terminal offers: with
    /// the `pixel` feature enabled and a protocol detected
    /// ([`Graphics::detect`](crate::pixel::Graphics::detect)), the plot panel
    /// becomes a real image; everywhere else — pipes, unknown terminals, tmux,
    /// or without the feature — exactly [`Plot::render`]. The one-call top of
    /// the resolution ladder for CLIs that already know their frame.
    ///
    /// Unlike [`Plot::render`] this consults the environment, so it is not
    /// deterministic across terminals; keep `render` for tests and snapshots.
    ///
    /// ```no_run
    /// let plot = malevich::line(&[1.0, 3.0, 2.0][..]);
    /// println!("{}", plot.render_best(&malevich::Frame::detect()));
    /// ```
    pub fn render_best(&self, frame: &Frame) -> String {
        #[cfg(feature = "pixel")]
        if let Some(graphics) = crate::pixel::Graphics::detect() {
            return self.render_pixels(frame, &graphics);
        }
        self.render(frame)
    }

    /// The fallible counterpart of [`Plot::render_best`]. It validates the plot
    /// and returns geometry/allocation errors instead of degrading to empty output.
    pub fn try_render_best(&self, frame: &Frame) -> crate::Result<String> {
        #[cfg(feature = "pixel")]
        if let Some(graphics) = crate::pixel::Graphics::detect() {
            return self.try_render_pixels(frame, &graphics);
        }
        self.try_render(frame)
    }

    /// Renders with the plot panel as a real image (feature `pixel`): chrome —
    /// title, axes, tick labels, legend — as text cells exactly like
    /// [`Plot::render`], and the plot rectangle as device-pixel graphics in the
    /// protocol `graphics` names. The returned string weaves both with relative
    /// cursor movement; print it to a terminal that speaks the protocol.
    ///
    /// Deterministic like every render path: no terminal is touched, and the
    /// same plot, frame, and graphics always produce the same string.
    #[cfg(feature = "pixel")]
    pub fn render_pixels(&self, frame: &Frame, graphics: &crate::pixel::Graphics) -> String {
        crate::pixel::render(self, frame, graphics, 0)
    }

    /// Validates and renders a hybrid pixel plot, returning bounded geometry or
    /// allocation failures as typed errors.
    #[cfg(feature = "pixel")]
    pub fn try_render_pixels(
        &self,
        frame: &Frame,
        graphics: &crate::pixel::Graphics,
    ) -> crate::Result<String> {
        self.validate()?;
        crate::pixel::try_render(self, frame, graphics, 0)
    }

    /// [`Plot::render_pixels`], anchored `column` cells from the left edge:
    /// every text row and the image cursor walk start with an absolute-column
    /// jump, so printing the block leaves anything to its left untouched. For
    /// hosts pasting plots side by side — print the left content, move the
    /// cursor back to the block's top row, print this. Rows stay relative;
    /// only columns are absolute, so scrollback is safe.
    #[cfg(feature = "pixel")]
    pub fn render_pixels_at(
        &self,
        frame: &Frame,
        graphics: &crate::pixel::Graphics,
        column: usize,
    ) -> String {
        crate::pixel::render(self, frame, graphics, column)
    }

    /// The fallible counterpart of [`Plot::render_pixels_at`].
    #[cfg(feature = "pixel")]
    pub fn try_render_pixels_at(
        &self,
        frame: &Frame,
        graphics: &crate::pixel::Graphics,
        column: usize,
    ) -> crate::Result<String> {
        self.validate()?;
        crate::pixel::try_render(self, frame, graphics, column)
    }

    /// Rasterizes for hybrid pixel output: chrome on a cell surface, marks on a
    /// device-pixel canvas at `cell` pixels per cell, one shared layout — so the
    /// scales map into device pixels and M4 buckets per pixel column.
    #[cfg(feature = "pixel")]
    pub(crate) fn try_rasterize_hybrid(
        &self,
        frame: &Frame,
        cell: (usize, usize),
    ) -> crate::Result<(Surface, crate::pixel::PixelCanvas, crate::render::PlotRect)> {
        use super::resolve::Reduce;
        use crate::mark::LineStyle;
        use crate::plot::resolve::{Kind, ResolvedLayer};
        use crate::render::PlotRect;

        let mut surface = Surface::try_new(frame.width, frame.height, frame.charset)?;
        let canvas = crate::pixel::PixelCanvas::try_new(frame.width, frame.height, cell)?;
        let empty = PlotRect {
            gutter: 0,
            top: 0,
            columns: 0,
            rows: 0,
        };
        if frame.width == 0 || frame.height == 0 || cell.0 == 0 || cell.1 == 0 {
            return Ok((surface, canvas, empty));
        }
        let mut canvas = canvas;
        let sample_width =
            frame
                .width
                .checked_mul(cell.0)
                .ok_or(crate::Error::DimensionTooLarge {
                    what: "pixel sample width",
                    requested: usize::MAX,
                    limit: crate::render::MAX_DEVICE_PIXELS,
                })?;
        let title = self.title.is_some();
        let scales = (&self.x, &self.y);
        let labels = (self.x_label.as_deref(), self.y_label.as_deref());
        let domains = (self.x_domain, self.y_domain);
        let palette = &frame.theme.palette;

        // The same probe-then-reduce dance as cell output (see rasterize_with):
        // M4 must bucket by the rendered column, which here is one device pixel.
        let probe = super::resolve::resolve(&self.layers, sample_width, palette, Reduce::Extent);
        let geometry = super::layout::Layout::compute(
            frame,
            cell,
            &probe,
            title,
            scales,
            labels,
            domains,
            self.colorbar,
        );
        let reduce = Reduce::Mapped {
            map: geometry.x_scale,
            columns: geometry.plot_sub_w,
        };
        let mut layers = super::resolve::resolve(&self.layers, sample_width, palette, reduce);
        // Corners is cell-glyph art; at pixel resolution the honest line is the
        // line itself.
        for layer in &mut layers {
            if let ResolvedLayer::Series { kind, .. } = layer
                && matches!(kind, Kind::Line(LineStyle::Corners))
            {
                *kind = Kind::Line(LineStyle::Pixels);
            }
        }
        let layout = super::layout::Layout::compute(
            frame,
            cell,
            &layers,
            title,
            scales,
            labels,
            domains,
            self.colorbar,
        );
        super::chrome::draw(
            &mut surface,
            &layout,
            self.title.as_deref(),
            labels,
            &layers,
        );
        super::draw::layers(&mut canvas, &layout, &layers);
        let rect = PlotRect {
            gutter: layout.gutter,
            top: layout.plot_top,
            columns: layout.plot_cols,
            rows: layout.plot_rows,
        };
        Ok((surface, canvas, rect))
    }

    #[cfg_attr(
        not(any(test, feature = "evcxr", feature = "ratatui")),
        allow(dead_code)
    )]
    pub(crate) fn rasterize(&self, frame: &Frame) -> Surface {
        self.try_rasterize(frame)
            .unwrap_or_else(|_| Surface::new(0, 0, frame.charset))
    }

    pub(crate) fn try_rasterize(&self, frame: &Frame) -> crate::Result<Surface> {
        self.try_rasterize_with(frame, true)
    }

    /// Rasterizes with M4 line downsampling optionally disabled. With `downsample`
    /// false, large line layers draw every point — the raw raster that M4 must
    /// reproduce, used as a test oracle for the aggregate-to-raster claim.
    #[cfg(test)]
    pub(crate) fn rasterize_with(&self, frame: &Frame, downsample: bool) -> Surface {
        self.try_rasterize_with(frame, downsample)
            .unwrap_or_else(|_| Surface::new(0, 0, frame.charset))
    }

    fn try_rasterize_with(&self, frame: &Frame, downsample: bool) -> crate::Result<Surface> {
        use super::resolve::Reduce;

        let mut surface = Surface::try_new(frame.width, frame.height, frame.charset)?;
        if frame.width == 0 || frame.height == 0 {
            return Ok(surface);
        }
        let (px, _) = frame.charset.pixels_per_cell();
        let sample_width = frame
            .width
            .checked_mul(px)
            .ok_or(crate::Error::DimensionTooLarge {
                what: "cell sample width",
                requested: usize::MAX,
                limit: crate::render::MAX_DEVICE_PIXELS,
            })?;
        let title = self.title.is_some();
        let scales = (&self.x, &self.y);
        let labels = (self.x_label.as_deref(), self.y_label.as_deref());
        let domains = (self.x_domain, self.y_domain);
        let palette = &frame.theme.palette;

        // Pixel-exact M4 must bucket by the *rendered* column, which the layout
        // fixes — but the layout needs the data first. So probe once with a coarse
        // reduction (M4 preserves the extents the layout reads), lift the scale and
        // width from that geometry, then reduce for real in exactly that raster space.
        let reduce = if downsample {
            let probe =
                super::resolve::resolve(&self.layers, sample_width, palette, Reduce::Extent);
            let geometry = super::layout::Layout::compute(
                frame,
                frame.charset.pixels_per_cell(),
                &probe,
                title,
                scales,
                labels,
                domains,
                self.colorbar,
            );
            Reduce::Mapped {
                map: geometry.x_scale,
                columns: geometry.plot_sub_w,
            }
        } else {
            Reduce::None
        };

        let layers = super::resolve::resolve(&self.layers, sample_width, palette, reduce);
        let layout = super::layout::Layout::compute(
            frame,
            frame.charset.pixels_per_cell(),
            &layers,
            title,
            scales,
            labels,
            domains,
            self.colorbar,
        );
        super::chrome::draw(
            &mut surface,
            &layout,
            self.title.as_deref(),
            labels,
            &layers,
        );
        super::draw::layers(&mut surface, &layout, &layers);
        Ok(surface)
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
