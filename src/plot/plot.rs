//! `Plot`: the retained chart description, and its resolve → layout → rasterize
//! pipeline.

use std::borrow::Cow;

use super::frame::Frame;
use crate::mark::{Mark, Source};
use crate::render::{Color, Surface, display_width, fit_width};
use crate::scale::{Linear, Ticks};

/// Layer colors when none are set explicitly. A single layer uses the terminal's
/// default foreground instead.
const PALETTE: [Color; 6] = [
    Color::Cyan,
    Color::Yellow,
    Color::Green,
    Color::Magenta,
    Color::Blue,
    Color::Red,
];

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
}

impl<'a> Plot<'a> {
    /// An empty plot with no layers and no furniture.
    pub fn new() -> Plot<'a> {
        Plot {
            layers: Vec::new(),
            title: None,
        }
    }

    /// Adds a mark as the next layer. Layers share scales: domains are the union of
    /// all layers' data, resolved at render time.
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
        let (px, py) = frame.charset.pixels_per_cell();

        let polylines = self.resolve(frame.width * px);
        let x_data = union(polylines.iter().map(|line| extent(&line.x))).unwrap_or((0.0, 1.0));
        let y_data = union(polylines.iter().map(|line| extent(&line.y))).unwrap_or((0.0, 1.0));

        // Vertical layout: title, plot rows, then the x axis and its labels — shed
        // in reverse priority when the frame is too short.
        let title_rows = usize::from(self.title.is_some() && frame.height >= 6);
        let axis_rows = match frame.height - title_rows {
            0..=1 => 0,
            2..=3 => 1,
            _ => 2,
        };
        let plot_rows = frame.height - title_rows - axis_rows;

        // Horizontal layout: the y-label gutter is measured, not fixed — and shed
        // entirely when it would eat the plot.
        let target = (plot_rows / 2).clamp(2, 8);
        let y_ticks = Ticks::linear(y_data.0, y_data.1, target);
        let mut label_width = y_ticks
            .iter()
            .map(|tick| display_width(&tick.label))
            .max()
            .unwrap_or(0);
        let mut gutter = label_width + 2;
        if gutter + 4 > frame.width {
            label_width = 0;
            gutter = usize::from(frame.width >= 2);
        }
        let plot_cols = frame.width - gutter;

        let y_domain = domain_with_ticks(y_data, &y_ticks);
        let plot_sub_w = (plot_cols * px).max(1);
        let plot_sub_h = (plot_rows * py).max(1);

        // The x axis gets ticks only when there is a row to label; the densest
        // labeling that fits without collisions wins.
        let labels_row_exists = axis_rows == 2;
        let x_ticks = if labels_row_exists {
            fit_x_ticks(x_data, plot_cols, plot_sub_w, px, gutter, frame.width)
        } else {
            None
        };
        let x_domain = match &x_ticks {
            Some(ticks) => domain_with_ticks(x_data, ticks),
            None => x_data,
        };

        let x_scale = Linear::new(x_domain, (0.0, (plot_sub_w - 1) as f64));
        let y_scale = Linear::new(y_domain, ((plot_sub_h - 1) as f64, 0.0));

        // Chrome first, marks last: marks own the plot area, chrome owns the rest.
        if title_rows == 1
            && let Some(title) = &self.title
        {
            let title = fit_width(title, frame.width);
            let len = display_width(&title) as i64;
            let start = ((frame.width as i64 - len) / 2).max(0);
            surface.text(start, 0, &title, Color::Default);
        }

        let plot_top = title_rows;
        if gutter >= 1 {
            let axis_column = (gutter - 1) as i64;
            for row in 0..plot_rows {
                surface.text(
                    axis_column,
                    (plot_top + row) as i64,
                    "\u{2502}",
                    Color::Default,
                );
            }
            if label_width > 0 {
                let mut used = vec![false; plot_rows];
                for tick in &y_ticks {
                    let sub = y_scale.map(tick.value);
                    if !sub.is_finite() {
                        continue;
                    }
                    let row = (sub.round() as usize) / py;
                    if row >= plot_rows || used[row] {
                        continue;
                    }
                    used[row] = true;
                    let cell_row = (plot_top + row) as i64;
                    let start = label_width as i64 - display_width(&tick.label) as i64;
                    surface.text(start, cell_row, &tick.label, Color::Default);
                    surface.text(axis_column, cell_row, "\u{2524}", Color::Default);
                }
            }
        }

        if axis_rows >= 1 {
            let axis_row = (plot_top + plot_rows) as i64;
            if gutter >= 1 {
                surface.text((gutter - 1) as i64, axis_row, "\u{2514}", Color::Default);
            }
            for col in 0..plot_cols {
                surface.text((gutter + col) as i64, axis_row, "\u{2500}", Color::Default);
            }
            if let Some(ticks) = &x_ticks {
                for tick in ticks {
                    let column = (x_scale.map(tick.value).round() as usize) / px;
                    surface.text(
                        (gutter + column) as i64,
                        axis_row,
                        "\u{252C}",
                        Color::Default,
                    );
                    let len = display_width(&tick.label) as i64;
                    let center = (gutter + column) as i64;
                    let start = (center - len / 2).clamp(0, (frame.width as i64 - len).max(0));
                    surface.text(start, axis_row + 1, &tick.label, Color::Default);
                }
            }
        }

        let x_offset = (gutter * px) as f64;
        let y_offset = (plot_top * py) as f64;
        for layer in &polylines {
            match layer.kind {
                Kind::Line => {
                    let mut previous: Option<(f64, f64)> = None;
                    for (&xv, &yv) in layer.x.iter().zip(layer.y.iter()) {
                        if !xv.is_finite() || !yv.is_finite() {
                            previous = None;
                            continue;
                        }
                        let position = (x_offset + x_scale.map(xv), y_offset + y_scale.map(yv));
                        match previous {
                            Some(from) => surface.line(from, position, layer.color),
                            None => surface.dot(position.0, position.1, layer.color),
                        }
                        previous = Some(position);
                    }
                }
                Kind::Points => {
                    for (&xv, &yv) in layer.x.iter().zip(layer.y.iter()) {
                        if xv.is_finite() && yv.is_finite() {
                            surface.dot(
                                x_offset + x_scale.map(xv),
                                y_offset + y_scale.map(yv),
                                layer.color,
                            );
                        }
                    }
                }
            }
        }

        surface
    }

    /// Materializes every layer into x/y columns plus a resolved color and drawing
    /// kind. Functions are sampled here, once per subpixel column of the frame width.
    fn resolve(&self, sample_width: usize) -> Vec<ResolvedLayer<'_>> {
        let single = self.layers.len() == 1;
        self.layers
            .iter()
            .enumerate()
            .map(|(index, mark)| {
                let assigned = |explicit: Option<Color>| {
                    explicit.unwrap_or(if single {
                        Color::Default
                    } else {
                        PALETTE[index % PALETTE.len()]
                    })
                };
                match mark {
                    Mark::Line(line) => {
                        let color = assigned(line.color);
                        match &line.source {
                            Source::Points { x, y } => ResolvedLayer {
                                x: index_or_borrow(x.as_ref(), y.len()),
                                y: Cow::Borrowed(y.as_slice()),
                                color,
                                kind: Kind::Line,
                            },
                            Source::Function { domain, function } => {
                                let samples = sample_width.max(2);
                                let step = (domain.1 - domain.0) / (samples - 1) as f64;
                                let x: Vec<f64> =
                                    (0..samples).map(|i| domain.0 + i as f64 * step).collect();
                                let y: Vec<f64> = x.iter().map(|&value| function(value)).collect();
                                ResolvedLayer {
                                    x: Cow::Owned(x),
                                    y: Cow::Owned(y),
                                    color,
                                    kind: Kind::Line,
                                }
                            }
                        }
                    }
                    Mark::Points(points) => ResolvedLayer {
                        x: index_or_borrow(points.x.as_ref(), points.y.len()),
                        y: Cow::Borrowed(points.y.as_slice()),
                        color: assigned(points.color),
                        kind: Kind::Points,
                    },
                }
            })
            .collect()
    }
}

/// The x channel: a borrowed series, or generated indices `0, 1, 2, …`.
fn index_or_borrow<'p>(x: Option<&'p crate::data::Series<'_>>, len: usize) -> Cow<'p, [f64]> {
    match x {
        Some(series) => Cow::Borrowed(series.as_slice()),
        None => Cow::Owned((0..len).map(|i| i as f64).collect()),
    }
}

impl std::fmt::Display for Plot<'_> {
    /// Renders with [`Frame::detect`]: the one-line `println!("{plot}")` path.
    /// Detection assumes stdout; for full control use [`Plot::render`].
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render(&Frame::detect()))
    }
}

/// How a resolved layer draws its columns.
enum Kind {
    Line,
    Points,
}

/// One layer, resolved to drawable columns.
struct ResolvedLayer<'p> {
    x: Cow<'p, [f64]>,
    y: Cow<'p, [f64]>,
    color: Color,
    kind: Kind,
}

/// The finite `(min, max)` of a column, or `None` without finite values.
fn extent(values: &[f64]) -> Option<(f64, f64)> {
    let mut extent: Option<(f64, f64)> = None;
    for &value in values.iter().filter(|value| value.is_finite()) {
        extent = match extent {
            None => Some((value, value)),
            Some((min, max)) => Some((min.min(value), max.max(value))),
        };
    }
    extent
}

/// Unions the extents of several columns.
fn union(extents: impl Iterator<Item = Option<(f64, f64)>>) -> Option<(f64, f64)> {
    extents
        .flatten()
        .reduce(|(min_a, max_a), (min_b, max_b)| (min_a.min(min_b), max_a.max(max_b)))
}

/// The axis domain: the data extent stretched to include the chosen ticks.
fn domain_with_ticks(data: (f64, f64), ticks: &Ticks) -> (f64, f64) {
    match (ticks.as_slice().first(), ticks.as_slice().last()) {
        (Some(first), Some(last)) => (data.0.min(first.value), data.1.max(last.value)),
        _ => data,
    }
}

/// Chooses the densest x labeling whose labels fit without collisions: centered
/// under their ticks, clamped to the frame, at least two cells apart.
fn fit_x_ticks(
    data: (f64, f64),
    plot_cols: usize,
    plot_sub_w: usize,
    px: usize,
    gutter: usize,
    frame_width: usize,
) -> Option<Ticks> {
    let densest = (plot_cols / 8).clamp(2, 12);
    for target in (2..=densest).rev() {
        let ticks = Ticks::linear(data.0, data.1, target);
        let domain = domain_with_ticks(data, &ticks);
        let scale = Linear::new(domain, (0.0, (plot_sub_w - 1) as f64));
        let mut last_end: i64 = i64::MIN;
        let mut fits = true;
        for tick in &ticks {
            let column = (scale.map(tick.value).round() as usize) / px;
            let len = display_width(&tick.label) as i64;
            let center = (gutter + column) as i64;
            let start = (center - len / 2).clamp(0, (frame_width as i64 - len).max(0));
            if start < last_end + 2 {
                fits = false;
                break;
            }
            last_end = start + len;
        }
        if fits {
            return Some(ticks);
        }
    }
    None
}

#[cfg(test)]
#[path = "tests/plot_tests.rs"]
mod tests;
