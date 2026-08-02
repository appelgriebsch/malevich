//! `Plot`: the retained chart description, and its resolve → layout → rasterize
//! pipeline.

use std::borrow::Cow;

use super::frame::Frame;
use crate::mark::{Mark, Source};
use crate::render::{Charset, Color, Surface, display_width, fit_width};
use crate::scale::{Band, Linear, Ticks};

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

        let layers = self.resolve(frame.width * px, &frame.theme.palette);
        let categories: Option<&[String]> = layers.iter().find_map(|layer| match layer {
            ResolvedLayer::Bars { categories, .. } if !categories.is_empty() => Some(*categories),
            _ => None,
        });

        let x_data = union(layers.iter().map(ResolvedLayer::x_extent)).unwrap_or((0.0, 1.0));
        let mut y_data = union(layers.iter().map(ResolvedLayer::y_extent)).unwrap_or((0.0, 1.0));
        if categories.is_some() {
            // Bar length is the encoding, so the baseline must be in view.
            y_data = (y_data.0.min(0.0), y_data.1.max(0.0));
        }

        // Vertical layout: title, legend, plot rows, then the x axis and its
        // labels — shed in priority order (legend first) when the frame is short.
        let ascii = frame.charset == Charset::Ascii;
        let title_rows = usize::from(self.title.is_some() && frame.height >= 6);
        let has_legend = layers
            .iter()
            .any(|layer| layer.legend_entry(ascii).is_some());
        let legend_rows = usize::from(has_legend && frame.height >= 8);
        let chrome_top = title_rows + legend_rows;
        let axis_rows = match frame.height - chrome_top {
            0..=1 => 0,
            2..=3 => 1,
            _ => 2,
        };
        let plot_rows = frame.height - chrome_top - axis_rows;

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

        // The x axis: a band scale when a bars layer is present, ticks otherwise.
        let band = categories.map(|c| Band::new(c.len(), (0.0, (plot_sub_w - 1) as f64)));
        let x_ticks = if band.is_none() && axis_rows == 2 {
            fit_x_ticks(x_data, plot_cols, plot_sub_w, px, gutter, frame.width)
        } else {
            None
        };
        let x_domain = match (&band, &x_ticks) {
            (Some(band), _) => (0.0, (band.count() - 1) as f64),
            (None, Some(ticks)) => domain_with_ticks(x_data, ticks),
            (None, None) => x_data,
        };
        let x_range = match &band {
            Some(band) => (band.center(0), band.center(band.count() - 1)),
            None => (0.0, (plot_sub_w - 1) as f64),
        };
        let x_scale = Linear::new(x_domain, x_range);
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

        if legend_rows == 1 {
            let entries: Vec<_> = layers
                .iter()
                .filter_map(|layer| layer.legend_entry(ascii))
                .collect();
            let total: usize = entries
                .iter()
                .map(|(swatch, _, label)| display_width(swatch) + 1 + display_width(label))
                .sum::<usize>()
                + 2 * entries.len().saturating_sub(1);
            let mut column = ((frame.width as i64 - total as i64) / 2).max(0);
            let row = title_rows as i64;
            for (index, (swatch, color, label)) in entries.iter().enumerate() {
                if index > 0 {
                    column += 2;
                }
                surface.text(column, row, swatch, *color);
                column += display_width(swatch) as i64 + 1;
                surface.text(column, row, label, Color::Default);
                column += display_width(label) as i64;
            }
        }

        let plot_top = chrome_top;
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
            if axis_rows == 2
                && let (Some(band), Some(categories)) = (&band, categories)
            {
                let budget = ((band.step() / px as f64).round() as usize).max(2) - 1;
                for (index, category) in categories.iter().enumerate() {
                    let label = fit_width(category, budget);
                    let len = display_width(&label) as i64;
                    let center = gutter as i64 + (band.center(index) / px as f64).round() as i64;
                    let start = (center - len / 2).clamp(0, (frame.width as i64 - len).max(0));
                    surface.text(start, axis_row + 1, &label, Color::Default);
                }
            }
        }

        let x_offset = (gutter * px) as f64;
        let y_offset = (plot_top * py) as f64;
        for layer in &layers {
            match layer {
                ResolvedLayer::Series {
                    x, y, color, kind, ..
                } => {
                    draw_series(
                        &mut surface,
                        kind,
                        x,
                        y,
                        *color,
                        &x_scale,
                        &y_scale,
                        (x_offset, y_offset),
                    );
                }
                ResolvedLayer::Bars { values, color, .. } => {
                    if let Some(band) = &band {
                        draw_bars(
                            &mut surface,
                            band,
                            &y_scale,
                            values,
                            *color,
                            (gutter, plot_top, plot_rows),
                            (px, py),
                            frame.charset,
                        );
                    }
                }
            }
        }

        surface
    }

    /// Materializes every layer into drawable columns plus a resolved color.
    /// Functions are sampled here, once per subpixel column of the frame width.
    fn resolve(&self, sample_width: usize, palette: &[Color; 6]) -> Vec<ResolvedLayer<'_>> {
        let single = self.layers.len() == 1;
        self.layers
            .iter()
            .enumerate()
            .map(|(index, mark)| {
                let assigned = |explicit: Option<Color>| {
                    explicit.unwrap_or(if single {
                        Color::Default
                    } else {
                        palette[index % palette.len()]
                    })
                };
                match mark {
                    Mark::Line(line) => {
                        let color = assigned(line.color);
                        match &line.source {
                            Source::Points { x, y } => ResolvedLayer::Series {
                                x: index_or_borrow(x.as_ref(), y.len()),
                                y: Cow::Borrowed(y.as_slice()),
                                color,
                                kind: Kind::Line,
                                label: line.label.as_deref(),
                            },
                            Source::Function { domain, function } => {
                                let samples = sample_width.max(2);
                                let step = (domain.1 - domain.0) / (samples - 1) as f64;
                                let x: Vec<f64> =
                                    (0..samples).map(|i| domain.0 + i as f64 * step).collect();
                                let y: Vec<f64> = x.iter().map(|&value| function(value)).collect();
                                ResolvedLayer::Series {
                                    x: Cow::Owned(x),
                                    y: Cow::Owned(y),
                                    color,
                                    kind: Kind::Line,
                                    label: line.label.as_deref(),
                                }
                            }
                        }
                    }
                    Mark::Points(points) => ResolvedLayer::Series {
                        x: index_or_borrow(points.x.as_ref(), points.y.len()),
                        y: Cow::Borrowed(points.y.as_slice()),
                        color: assigned(points.color),
                        kind: Kind::Points,
                        label: points.label.as_deref(),
                    },
                    Mark::Bars(bars) => ResolvedLayer::Bars {
                        categories: &bars.categories,
                        values: bars.values.as_slice(),
                        color: assigned(bars.color),
                        label: bars.label.as_deref(),
                    },
                }
            })
            .collect()
    }
}

impl std::fmt::Display for Plot<'_> {
    /// Renders with [`Frame::detect`]: the one-line `println!("{plot}")` path.
    /// Detection assumes stdout; for full control use [`Plot::render`].
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render(&Frame::detect()))
    }
}

/// How a resolved series layer draws its columns.
enum Kind {
    Line,
    Points,
}

/// One layer, resolved to drawable data.
enum ResolvedLayer<'p> {
    Series {
        x: Cow<'p, [f64]>,
        y: Cow<'p, [f64]>,
        color: Color,
        kind: Kind,
        label: Option<&'p str>,
    },
    Bars {
        categories: &'p [String],
        values: &'p [f64],
        color: Color,
        label: Option<&'p str>,
    },
}

impl ResolvedLayer<'_> {
    /// The finite x extent this layer contributes to the shared domain.
    /// Bars contribute none — their axis is the band scale.
    fn x_extent(&self) -> Option<(f64, f64)> {
        match self {
            ResolvedLayer::Series { x, .. } => extent(x),
            ResolvedLayer::Bars { .. } => None,
        }
    }

    /// The finite y extent this layer contributes to the shared domain.
    fn y_extent(&self) -> Option<(f64, f64)> {
        match self {
            ResolvedLayer::Series { y, .. } => extent(y),
            ResolvedLayer::Bars { values, .. } => extent(values),
        }
    }

    /// The legend entry of this layer, if labeled: swatch text, color, label.
    fn legend_entry(&self, ascii: bool) -> Option<(&'static str, Color, &str)> {
        let (swatch, color, label) = match self {
            ResolvedLayer::Series {
                color, kind, label, ..
            } => {
                let swatch = match (kind, ascii) {
                    (Kind::Line, false) => "\u{2500}\u{2500}",
                    (Kind::Line, true) => "--",
                    (Kind::Points, false) => "\u{2022}\u{2022}",
                    (Kind::Points, true) => "**",
                };
                (swatch, *color, *label)
            }
            ResolvedLayer::Bars { color, label, .. } => {
                let swatch = if ascii { "##" } else { "\u{2588}\u{2588}" };
                (swatch, *color, *label)
            }
        };
        label.map(|label| (swatch, color, label))
    }
}

/// Draws one line or points layer through the shared scales.
#[allow(clippy::too_many_arguments)]
fn draw_series(
    surface: &mut Surface,
    kind: &Kind,
    x: &[f64],
    y: &[f64],
    color: Color,
    x_scale: &Linear,
    y_scale: &Linear,
    offset: (f64, f64),
) {
    match kind {
        Kind::Line => {
            let mut previous: Option<(f64, f64)> = None;
            for (&xv, &yv) in x.iter().zip(y.iter()) {
                if !xv.is_finite() || !yv.is_finite() {
                    previous = None;
                    continue;
                }
                let position = (offset.0 + x_scale.map(xv), offset.1 + y_scale.map(yv));
                match previous {
                    Some(from) => surface.line(from, position, color),
                    None => surface.dot(position.0, position.1, color),
                }
                previous = Some(position);
            }
        }
        Kind::Points => {
            for (&xv, &yv) in x.iter().zip(y.iter()) {
                if xv.is_finite() && yv.is_finite() {
                    surface.dot(
                        offset.0 + x_scale.map(xv),
                        offset.1 + y_scale.map(yv),
                        color,
                    );
                }
            }
        }
    }
}

/// Draws one bars layer: cell-aligned columns from the zero baseline, with
/// eighth-block partial fills at the value end (upward bars) or coarse upper-block
/// fills (downward bars — Unicode has no lower-anchored upper ramp).
#[allow(clippy::too_many_arguments)]
fn draw_bars(
    surface: &mut Surface,
    band: &Band,
    y_scale: &Linear,
    values: &[f64],
    color: Color,
    place: (usize, usize, usize),
    density: (usize, usize),
    charset: Charset,
) {
    let (gutter, plot_top, plot_rows) = place;
    let (px, py) = density;
    let ramp = charset.fill_ramp();
    let eighths = ramp.len() == 8;
    let baseline = y_scale.map(0.0) / py as f64;
    let mut buffer = [0u8; 4];

    for (index, &value) in values.iter().enumerate().take(band.count()) {
        if !value.is_finite() || value == 0.0 {
            continue;
        }
        let left = (band.position(index) / px as f64).round() as i64;
        let right =
            (((band.position(index) + band.bandwidth()) / px as f64).round() as i64).max(left + 1);
        let end = y_scale.map(value) / py as f64;

        for column in left..right {
            let cell_column = gutter as i64 + column;
            if value > 0.0 {
                // Upward: full cells from the (snapped-down) baseline, a
                // bottom-anchored partial at the top.
                let bottom = baseline.ceil().min(plot_rows as f64);
                let top = end.max(0.0);
                let mut row = top.floor();
                while row < bottom {
                    let coverage = ((row + 1.0 - top).min(1.0) * 8.0).round() as usize;
                    let glyph: Option<char> = if eighths {
                        (coverage >= 1).then(|| ramp[coverage.min(8) - 1])
                    } else {
                        (coverage >= 4).then(|| ramp[0])
                    };
                    if let Some(glyph) = glyph {
                        surface.text(
                            cell_column,
                            plot_top as i64 + row as i64,
                            glyph.encode_utf8(&mut buffer),
                            color,
                        );
                    }
                    row += 1.0;
                }
            } else {
                // Downward: full cells from the (snapped-up) baseline, a coarse
                // top-anchored partial at the bottom.
                let top = baseline.floor().max(0.0);
                let bottom = end.min(plot_rows as f64);
                let mut row = top;
                while row < bottom.ceil() {
                    let coverage = (bottom - row).min(1.0);
                    let glyph: Option<char> = if !eighths {
                        (coverage >= 0.5).then(|| ramp[0])
                    } else if coverage >= 7.0 / 8.0 {
                        Some('\u{2588}')
                    } else if coverage >= 0.5 {
                        Some('\u{2580}')
                    } else if coverage >= 1.0 / 8.0 {
                        Some('\u{2594}')
                    } else {
                        None
                    };
                    if let Some(glyph) = glyph {
                        surface.text(
                            cell_column,
                            plot_top as i64 + row as i64,
                            glyph.encode_utf8(&mut buffer),
                            color,
                        );
                    }
                    row += 1.0;
                }
            }
        }
    }
}

/// The x channel: a borrowed series, or generated indices `0, 1, 2, …`.
fn index_or_borrow<'p>(x: Option<&'p crate::data::Series<'_>>, len: usize) -> Cow<'p, [f64]> {
    match x {
        Some(series) => Cow::Borrowed(series.as_slice()),
        None => Cow::Owned((0..len).map(|i| i as f64).collect()),
    }
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
