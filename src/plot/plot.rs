//! `Plot`: the retained chart description, and its resolve → layout → rasterize
//! pipeline.

use std::borrow::Cow;

use super::frame::Frame;
use crate::mark::{Mark, Orientation, Placement, Source};
use crate::render::{Charset, Color, Surface, display_width, fit_width};
use crate::scale::{Band, Colormap, Linear, Ticks};

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
}

impl<'a> Plot<'a> {
    /// An empty plot with no layers and no furniture.
    pub fn new() -> Plot<'a> {
        Plot {
            layers: Vec::new(),
            title: None,
            log_x: false,
            log_y: false,
        }
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
            ResolvedLayer::Bars {
                placement: Placement::Bands(categories),
                ..
            } if !categories.is_empty() => Some(categories.as_slice()),
            _ => None,
        });
        let has_bars = layers
            .iter()
            .any(|layer| matches!(layer, ResolvedLayer::Bars { .. }));

        let log_x = self.log_x && categories.is_none();
        let log_y = self.log_y;
        let x_data = if log_x {
            union(layers.iter().map(ResolvedLayer::x_extent_positive)).unwrap_or((1.0, 100.0))
        } else {
            union(layers.iter().map(ResolvedLayer::x_extent)).unwrap_or((0.0, 1.0))
        };
        let mut y_data = if log_y {
            union(layers.iter().map(ResolvedLayer::y_extent_positive)).unwrap_or((1.0, 100.0))
        } else {
            union(layers.iter().map(ResolvedLayer::y_extent)).unwrap_or((0.0, 1.0))
        };
        if has_bars && !log_y {
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
        let y_ticks = if log_y {
            Ticks::log10(y_data.0, y_data.1, target)
        } else {
            Ticks::linear(y_data.0, y_data.1, target)
        };
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
            if log_x {
                Some(Ticks::log10(
                    x_data.0,
                    x_data.1,
                    (plot_cols / 10).clamp(2, 8),
                ))
            } else {
                fit_x_ticks(x_data, plot_cols, plot_sub_w, px, gutter, frame.width)
            }
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
        let x_scale = Map::build(x_domain, x_range, log_x);
        let y_scale = Map::build(y_domain, ((plot_sub_h - 1) as f64, 0.0), log_y);

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
                ResolvedLayer::Area {
                    x,
                    low,
                    high,
                    color,
                    ..
                } => {
                    draw_area(
                        &mut surface,
                        x,
                        *low,
                        high,
                        *color,
                        &x_scale,
                        &y_scale,
                        (x_offset, y_offset),
                    );
                }
                ResolvedLayer::Cells {
                    columns,
                    values,
                    extents,
                    colormap,
                } => {
                    draw_cells(
                        &mut surface,
                        *columns,
                        values,
                        *extents,
                        *colormap,
                        &x_scale,
                        &y_scale,
                        (gutter, plot_top, plot_cols, plot_rows),
                        (px, py),
                    );
                }
                ResolvedLayer::Rule {
                    orientation, color, ..
                } => match orientation {
                    Orientation::Horizontal(y) => {
                        let sy = y_offset + y_scale.map(*y);
                        surface.line(
                            (x_offset, sy),
                            (x_offset + (plot_sub_w - 1) as f64, sy),
                            *color,
                        );
                    }
                    Orientation::Vertical(x) => {
                        let sx = x_offset + x_scale.map(*x);
                        surface.line(
                            (sx, y_offset),
                            (sx, y_offset + (plot_sub_h - 1) as f64),
                            *color,
                        );
                    }
                },
                ResolvedLayer::Text { x, y, text, color } => {
                    let sx = x_offset + x_scale.map(*x);
                    let sy = y_offset + y_scale.map(*y);
                    if sx.is_finite() && sy.is_finite() {
                        surface.text(
                            (sx / px as f64).round() as i64,
                            (sy / py as f64).round() as i64,
                            text,
                            *color,
                        );
                    }
                }
                ResolvedLayer::Bars {
                    placement,
                    values,
                    color,
                    ..
                } => match placement {
                    Placement::Bands(_) => {
                        if let Some(band) = &band {
                            draw_bars(
                                &mut surface,
                                &|index| {
                                    (
                                        band.position(index),
                                        band.position(index) + band.bandwidth(),
                                    )
                                },
                                &y_scale,
                                values,
                                *color,
                                (gutter, plot_top, plot_rows),
                                (px, py),
                                frame.charset,
                            );
                        }
                    }
                    Placement::Spans { start, width } => {
                        draw_bars(
                            &mut surface,
                            &|index| {
                                let left = x_scale.map(start + width * index as f64);
                                let right = x_scale.map(start + width * (index + 1) as f64);
                                (left, right)
                            },
                            &y_scale,
                            values,
                            *color,
                            (gutter, plot_top, plot_rows),
                            (px, py),
                            frame.charset,
                        );
                    }
                },
            }
        }

        surface
    }

    /// Materializes every layer into drawable columns plus a resolved color.
    /// Functions are sampled here, once per subpixel column of the frame width.
    fn resolve(&self, sample_width: usize, palette: &[Color; 6]) -> Vec<ResolvedLayer<'_>> {
        // Annotations (rules, text) draw in the default foreground and do not
        // consume palette slots; a single data layer draws in the default too.
        let data_layers = self
            .layers
            .iter()
            .filter(|mark| !matches!(mark, Mark::Rule(_) | Mark::Text(_)))
            .count();
        let single = data_layers == 1;
        let mut palette_index = 0usize;
        self.layers
            .iter()
            .map(|mark| {
                let mut assigned = |explicit: Option<Color>| {
                    let index = palette_index;
                    palette_index += 1;
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
                            Source::Points { x, y } => {
                                // The aggregate-to-raster pipeline: past four points
                                // per subpixel column, M4 reduces the series with
                                // zero visual error. Non-monotonic x declines.
                                let downsampled = if y.len() > 4 * sample_width.max(1) {
                                    match x {
                                        Some(series) => crate::stat::m4(
                                            series.as_slice(),
                                            y.as_slice(),
                                            sample_width,
                                        ),
                                        None => crate::stat::m4_indexed(y.as_slice(), sample_width),
                                    }
                                } else {
                                    None
                                };
                                match downsampled {
                                    Some((dx, dy)) => ResolvedLayer::Series {
                                        x: Cow::Owned(dx),
                                        y: Cow::Owned(dy),
                                        color,
                                        kind: Kind::Line,
                                        label: line.label.as_deref(),
                                    },
                                    None => ResolvedLayer::Series {
                                        x: index_or_borrow(x.as_ref(), y.len()),
                                        y: Cow::Borrowed(y.as_slice()),
                                        color,
                                        kind: Kind::Line,
                                        label: line.label.as_deref(),
                                    },
                                }
                            }
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
                        placement: &bars.placement,
                        values: bars.values.as_slice(),
                        color: assigned(bars.color),
                        label: bars.label.as_deref(),
                    },
                    Mark::Area(area) => ResolvedLayer::Area {
                        x: index_or_borrow(area.x.as_ref(), area.high.len()),
                        low: area.low.as_ref().map(|series| series.as_slice()),
                        high: area.high.as_slice(),
                        color: assigned(area.color),
                        label: area.label.as_deref(),
                    },
                    Mark::Cells(cells) => ResolvedLayer::Cells {
                        columns: cells.columns,
                        values: cells.values.as_slice(),
                        extents: cells.extents,
                        colormap: cells.colormap,
                    },
                    Mark::Rule(rule) => ResolvedLayer::Rule {
                        orientation: rule.orientation,
                        color: rule.color.unwrap_or(Color::Default),
                        label: rule.label.as_deref(),
                    },
                    Mark::Text(text) => ResolvedLayer::Text {
                        x: text.x,
                        y: text.y,
                        text: &text.text,
                        color: text.color.unwrap_or(Color::Default),
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

/// A position scale resolved for drawing: linear, or linear over `log10`.
///
/// The log arm maps `value.log10()`, so zero and negative values become `NaN` — and
/// `NaN` is already the gap encoding, which is exactly the honest behavior.
#[derive(Debug, Clone, Copy)]
enum Map {
    Linear(Linear),
    Log(Linear),
}

impl Map {
    fn build(domain: (f64, f64), range: (f64, f64), log: bool) -> Map {
        if log {
            Map::Log(Linear::new((domain.0.log10(), domain.1.log10()), range))
        } else {
            Map::Linear(Linear::new(domain, range))
        }
    }

    fn map(&self, value: f64) -> f64 {
        match self {
            Map::Linear(linear) => linear.map(value),
            Map::Log(linear) => linear.map(value.log10()),
        }
    }
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
        placement: &'p Placement,
        values: &'p [f64],
        color: Color,
        label: Option<&'p str>,
    },
    Area {
        x: Cow<'p, [f64]>,
        low: Option<&'p [f64]>,
        high: &'p [f64],
        color: Color,
        label: Option<&'p str>,
    },
    Cells {
        columns: usize,
        values: &'p [f64],
        extents: Option<((f64, f64), (f64, f64))>,
        colormap: Colormap,
    },
    Rule {
        orientation: Orientation,
        color: Color,
        label: Option<&'p str>,
    },
    Text {
        x: f64,
        y: f64,
        text: &'p str,
        color: Color,
    },
}

impl ResolvedLayer<'_> {
    /// The finite x extent this layer contributes to the shared domain.
    /// Bars contribute none — their axis is the band scale.
    fn x_extent(&self) -> Option<(f64, f64)> {
        match self {
            ResolvedLayer::Series { x, .. } => extent(x),
            ResolvedLayer::Bars {
                placement: Placement::Spans { start, width },
                values,
                ..
            } => Some((*start, start + width * values.len() as f64)),
            ResolvedLayer::Bars { .. } => None,
            ResolvedLayer::Area { x, .. } => extent(x),
            ResolvedLayer::Rule {
                orientation: Orientation::Vertical(x),
                ..
            } => Some((*x, *x)),
            ResolvedLayer::Rule { .. } => None,
            ResolvedLayer::Text { x, .. } => Some((*x, *x)),
            ResolvedLayer::Cells {
                columns, extents, ..
            } => Some(match extents {
                Some((x, _)) => *x,
                None => (0.0, *columns as f64),
            }),
        }
    }

    /// The finite y extent this layer contributes to the shared domain.
    fn y_extent(&self) -> Option<(f64, f64)> {
        match self {
            ResolvedLayer::Series { y, .. } => extent(y),
            ResolvedLayer::Bars { values, .. } => extent(values),
            ResolvedLayer::Area { low, high, .. } => {
                let highs = extent(high);
                let lows = match low {
                    Some(low) => extent(low),
                    // A baseline fill keeps zero in view, like bars.
                    None => Some((0.0, 0.0)),
                };
                union([highs, lows].into_iter())
            }
            ResolvedLayer::Rule {
                orientation: Orientation::Horizontal(y),
                ..
            } => Some((*y, *y)),
            ResolvedLayer::Rule { .. } => None,
            ResolvedLayer::Text { y, .. } => Some((*y, *y)),
            ResolvedLayer::Cells {
                columns,
                values,
                extents,
                ..
            } => Some(match extents {
                Some((_, y)) => *y,
                None => (0.0, (values.len() / (*columns).max(1)) as f64),
            }),
        }
    }

    /// [`ResolvedLayer::x_extent`] over strictly positive values (log axes).
    fn x_extent_positive(&self) -> Option<(f64, f64)> {
        match self {
            ResolvedLayer::Series { x, .. } => extent_positive(x),
            ResolvedLayer::Area { x, .. } => extent_positive(x),
            ResolvedLayer::Rule {
                orientation: Orientation::Vertical(x),
                ..
            } if *x > 0.0 => Some((*x, *x)),
            ResolvedLayer::Text { x, .. } if *x > 0.0 => Some((*x, *x)),
            ResolvedLayer::Cells { .. } => self.x_extent().filter(|(lo, _)| *lo > 0.0),
            _ => None,
        }
    }

    /// [`ResolvedLayer::y_extent`] over strictly positive values (log axes).
    fn y_extent_positive(&self) -> Option<(f64, f64)> {
        match self {
            ResolvedLayer::Series { y, .. } => extent_positive(y),
            ResolvedLayer::Bars { values, .. } => extent_positive(values),
            ResolvedLayer::Area { low, high, .. } => {
                let highs = extent_positive(high);
                let lows = low.and_then(extent_positive);
                union([highs, lows].into_iter())
            }
            ResolvedLayer::Rule {
                orientation: Orientation::Horizontal(y),
                ..
            } if *y > 0.0 => Some((*y, *y)),
            ResolvedLayer::Text { y, .. } if *y > 0.0 => Some((*y, *y)),
            ResolvedLayer::Cells { .. } => self.y_extent().filter(|(lo, _)| *lo > 0.0),
            _ => None,
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
            ResolvedLayer::Area { color, label, .. } => {
                let swatch = if ascii { "##" } else { "\u{2584}\u{2584}" };
                (swatch, *color, *label)
            }
            ResolvedLayer::Rule { color, label, .. } => {
                let swatch = if ascii { "--" } else { "\u{2500}\u{2500}" };
                (swatch, *color, *label)
            }
            ResolvedLayer::Text { .. } | ResolvedLayer::Cells { .. } => return None,
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
    x_scale: &Map,
    y_scale: &Map,
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
    span: &dyn Fn(usize) -> (f64, f64),
    y_scale: &Map,
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

    for (index, &value) in values.iter().enumerate() {
        if !value.is_finite() || value == 0.0 {
            continue;
        }
        let (left_sub, right_sub) = span(index);
        if !left_sub.is_finite() || !right_sub.is_finite() {
            continue;
        }
        let left = (left_sub / px as f64).round() as i64;
        let right = ((right_sub / px as f64).round() as i64).max(left + 1);
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

/// Draws one cells layer: for every surface cell inside the plot area, the nearest
/// grid sample renders as a shade-ramp glyph colored by the colormap — value in
/// glyph and color both, readable at every color tier. Gaps stay blank.
#[allow(clippy::too_many_arguments)]
fn draw_cells(
    surface: &mut Surface,
    columns: usize,
    values: &[f64],
    extents: Option<((f64, f64), (f64, f64))>,
    colormap: Colormap,
    x_scale: &Map,
    y_scale: &Map,
    place: (usize, usize, usize, usize),
    density: (usize, usize),
) {
    const RAMP: [char; 4] = ['\u{2591}', '\u{2592}', '\u{2593}', '\u{2588}'];
    let (gutter, plot_top, plot_cols, plot_rows) = place;
    let (px, py) = density;
    let rows = values.len() / columns.max(1);
    if rows == 0 {
        return;
    }
    let Some((low, high)) = extent(values) else {
        return;
    };
    let spread = if high > low { high - low } else { 1.0 };
    let ((x0, x1), (y0, y1)) = extents.unwrap_or(((0.0, columns as f64), (0.0, rows as f64)));
    let mut buffer = [0u8; 4];

    for cell_row in 0..plot_rows {
        for cell_col in 0..plot_cols {
            // The data position at this cell's center, via the shared scales'
            // subpixel geometry.
            let sub_x = (cell_col * px) as f64 + px as f64 / 2.0;
            let sub_y = (cell_row * py) as f64 + py as f64 / 2.0;
            let fx = position_on(x_scale, sub_x, x0, x1);
            let fy = position_on(y_scale, sub_y, y0, y1);
            let (Some(fx), Some(fy)) = (fx, fy) else {
                continue;
            };
            let column = ((fx - x0) / (x1 - x0) * columns as f64).floor();
            let row = ((fy - y0) / (y1 - y0) * rows as f64).floor();
            if column < 0.0 || row < 0.0 {
                continue;
            }
            let (column, row) = (column as usize, row as usize);
            if column >= columns || row >= rows {
                continue;
            }
            let value = values[row * columns + column];
            if !value.is_finite() {
                continue;
            }
            let position = (value - low) / spread;
            let glyph = RAMP[((position * 4.0) as usize).min(3)];
            surface.text(
                (gutter + cell_col) as i64,
                (plot_top + cell_row) as i64,
                glyph.encode_utf8(&mut buffer),
                colormap.color(position),
            );
        }
    }
}

/// Inverts a scale at a subpixel position, returning the data value if it lands
/// inside `[lo, hi]`.
fn position_on(scale: &Map, sub: f64, lo: f64, hi: f64) -> Option<f64> {
    // Sample the scale forward at both ends to invert linearly in subpixel space —
    // exact for linear scales, and cells are not drawn on log axes.
    let s0 = scale.map(lo);
    let s1 = scale.map(hi);
    if !s0.is_finite() || !s1.is_finite() || s0 == s1 {
        return None;
    }
    let t = (sub - s0) / (s1 - s0);
    if !(0.0..1.0).contains(&t) {
        return None;
    }
    Some(lo + t * (hi - lo))
}

/// Draws one area layer: for every subpixel column a segment covers, a vertical
/// run between its interpolated low and high edges — solid in every charset, with
/// subpixel edge precision.
#[allow(clippy::too_many_arguments)]
fn draw_area(
    surface: &mut Surface,
    x: &[f64],
    low: Option<&[f64]>,
    high: &[f64],
    color: Color,
    x_scale: &Map,
    y_scale: &Map,
    offset: (f64, f64),
) {
    let mut previous: Option<(f64, f64, f64)> = None;
    for index in 0..high.len() {
        let xv = x[index];
        let hv = high[index];
        let lv = low.map_or(0.0, |low| low[index]);
        if !xv.is_finite() || !hv.is_finite() || !lv.is_finite() {
            previous = None;
            continue;
        }
        let sx = offset.0 + x_scale.map(xv);
        let sh = offset.1 + y_scale.map(hv);
        let sl = offset.1 + y_scale.map(lv);
        match previous {
            Some((px_, pl, ph)) => {
                let (from, to) = if px_ <= sx { (px_, sx) } else { (sx, px_) };
                let span = sx - px_;
                for column in (from.round() as i64)..=(to.round() as i64) {
                    let t = if span.abs() < f64::EPSILON {
                        0.0
                    } else {
                        ((column as f64 - px_) / span).clamp(0.0, 1.0)
                    };
                    let column_low = pl + (sl - pl) * t;
                    let column_high = ph + (sh - ph) * t;
                    surface.line(
                        (column as f64, column_low),
                        (column as f64, column_high),
                        color,
                    );
                }
            }
            None => surface.line((sx, sl), (sx, sh), color),
        }
        previous = Some((sx, sl, sh));
    }
}

/// The x channel: a borrowed series, or generated indices `0, 1, 2, …`.
fn index_or_borrow<'p>(x: Option<&'p crate::data::Series<'_>>, len: usize) -> Cow<'p, [f64]> {
    match x {
        Some(series) => Cow::Borrowed(series.as_slice()),
        None => Cow::Owned((0..len).map(|i| i as f64).collect()),
    }
}

/// The finite `(min, max)` over strictly positive values, or `None` without any.
fn extent_positive(values: &[f64]) -> Option<(f64, f64)> {
    let mut extent: Option<(f64, f64)> = None;
    for &value in values
        .iter()
        .filter(|value| value.is_finite() && **value > 0.0)
    {
        extent = match extent {
            None => Some((value, value)),
            Some((min, max)) => Some((min.min(value), max.max(value))),
        };
    }
    extent
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
