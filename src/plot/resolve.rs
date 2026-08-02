//! Resolution: marks materialized into drawable columns with resolved colors.

use std::borrow::Cow;

use crate::mark::{Mark, Orientation, Placement, RangePlacement, Source};
use crate::render::Color;
use crate::scale::Colormap;

/// How a resolved series layer draws its columns.
pub(crate) enum Kind {
    Line,
    Points,
}

/// One layer, resolved to drawable data.
pub(crate) enum ResolvedLayer<'p> {
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
        horizontal: bool,
        color: Color,
        label: Option<&'p str>,
    },
    Cells {
        columns: usize,
        values: &'p [f64],
        extents: Option<((f64, f64), (f64, f64))>,
        colormap: Colormap,
    },
    Range {
        x: Cow<'p, [f64]>,
        categories: Option<&'p [String]>,
        low: &'p [f64],
        high: &'p [f64],
        body: Option<(&'p [f64], &'p [f64])>,
        marker: Option<&'p [f64]>,
        color: Color,
        label: Option<&'p str>,
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
    pub(crate) fn x_extent(&self) -> Option<(f64, f64)> {
        match self {
            ResolvedLayer::Series { x, .. } => extent(x),
            ResolvedLayer::Bars {
                placement: Placement::Spans { start, width },
                values,
                ..
            } => Some((*start, start + width * values.len() as f64)),
            ResolvedLayer::Bars { .. } => None,
            ResolvedLayer::Area {
                x,
                low,
                high,
                horizontal,
                ..
            } => {
                if *horizontal {
                    union([low.and_then(extent), extent(high)].into_iter())
                } else {
                    extent(x)
                }
            }
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
            ResolvedLayer::Range { x, categories, .. } => {
                if categories.is_some() {
                    None
                } else {
                    extent(x)
                }
            }
        }
    }

    /// The finite y extent this layer contributes to the shared domain.
    pub(crate) fn y_extent(&self) -> Option<(f64, f64)> {
        match self {
            ResolvedLayer::Series { y, .. } => extent(y),
            ResolvedLayer::Bars { values, .. } => extent(values),
            ResolvedLayer::Area {
                x,
                low,
                high,
                horizontal,
                ..
            } => {
                if *horizontal {
                    extent(x)
                } else {
                    let highs = extent(high);
                    let lows = match low {
                        Some(low) => extent(low),
                        // A baseline fill keeps zero in view, like bars.
                        None => Some((0.0, 0.0)),
                    };
                    union([highs, lows].into_iter())
                }
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
            ResolvedLayer::Range {
                low, high, marker, ..
            } => union([extent(low), extent(high), marker.and_then(extent)].into_iter()),
        }
    }

    /// [`ResolvedLayer::x_extent`] over strictly positive values (log axes).
    pub(crate) fn x_extent_positive(&self) -> Option<(f64, f64)> {
        match self {
            ResolvedLayer::Series { x, .. } => extent_positive(x),
            ResolvedLayer::Area {
                x,
                low,
                high,
                horizontal,
                ..
            } => {
                if *horizontal {
                    union([low.and_then(extent_positive), extent_positive(high)].into_iter())
                } else {
                    extent_positive(x)
                }
            }
            ResolvedLayer::Rule {
                orientation: Orientation::Vertical(x),
                ..
            } if *x > 0.0 => Some((*x, *x)),
            ResolvedLayer::Text { x, .. } if *x > 0.0 => Some((*x, *x)),
            ResolvedLayer::Cells { .. } => self.x_extent().filter(|(lo, _)| *lo > 0.0),
            ResolvedLayer::Range {
                x,
                categories: None,
                ..
            } => extent_positive(x),
            _ => None,
        }
    }

    /// [`ResolvedLayer::y_extent`] over strictly positive values (log axes).
    pub(crate) fn y_extent_positive(&self) -> Option<(f64, f64)> {
        match self {
            ResolvedLayer::Series { y, .. } => extent_positive(y),
            ResolvedLayer::Bars { values, .. } => extent_positive(values),
            ResolvedLayer::Area {
                x,
                low,
                high,
                horizontal,
                ..
            } => {
                if *horizontal {
                    extent_positive(x)
                } else {
                    union([extent_positive(high), low.and_then(extent_positive)].into_iter())
                }
            }
            ResolvedLayer::Rule {
                orientation: Orientation::Horizontal(y),
                ..
            } if *y > 0.0 => Some((*y, *y)),
            ResolvedLayer::Text { y, .. } if *y > 0.0 => Some((*y, *y)),
            ResolvedLayer::Cells { .. } => self.y_extent().filter(|(lo, _)| *lo > 0.0),
            ResolvedLayer::Range { low, high, .. } => {
                union([extent_positive(low), extent_positive(high)].into_iter())
            }
            _ => None,
        }
    }

    /// The legend entry of this layer, if labeled: swatch text, color, label.
    pub(crate) fn legend_entry(&self, ascii: bool) -> Option<(&'static str, Color, &str)> {
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
            ResolvedLayer::Range { color, label, .. } => {
                let swatch = if ascii { "||" } else { "\u{2503}\u{2503}" };
                (swatch, *color, *label)
            }
            ResolvedLayer::Text { .. } | ResolvedLayer::Cells { .. } => return None,
        };
        label.map(|label| (swatch, color, label))
    }
}

/// Materializes every layer into drawable columns plus a resolved color.
/// Functions are sampled here, once per subpixel column of the frame width.
pub(crate) fn resolve<'p>(
    marks: &'p [Mark<'_>],
    sample_width: usize,
    palette: &[Color; 6],
) -> Vec<ResolvedLayer<'p>> {
    // Annotations (rules, text) draw in the default foreground and do not
    // consume palette slots; a single data layer draws in the default too.
    let data_layers = marks
        .iter()
        .filter(|mark| !matches!(mark, Mark::Rule(_) | Mark::Text(_)))
        .count();
    let single = data_layers == 1;
    let mut palette_index = 0usize;
    marks
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
                    horizontal: area.horizontal,
                    color: assigned(area.color),
                    label: area.label.as_deref(),
                },
                Mark::Cells(cells) => ResolvedLayer::Cells {
                    columns: cells.columns,
                    values: cells.values.as_slice(),
                    extents: cells.extents,
                    colormap: cells.colormap,
                },
                Mark::Range(range) => {
                    let (x, categories): (Cow<'_, [f64]>, _) = match &range.placement {
                        RangePlacement::Numeric(x) => {
                            (index_or_borrow(x.as_ref(), range.low.len()), None)
                        }
                        RangePlacement::Bands(categories) => (
                            Cow::Owned((0..categories.len()).map(|i| i as f64).collect()),
                            Some(categories.as_slice()),
                        ),
                    };
                    ResolvedLayer::Range {
                        x,
                        categories,
                        low: range.low.as_slice(),
                        high: range.high.as_slice(),
                        body: range
                            .body
                            .as_ref()
                            .map(|(low, high)| (low.as_slice(), high.as_slice())),
                        marker: range.marker.as_ref().map(|m| m.as_slice()),
                        color: assigned(range.color),
                        label: range.label.as_deref(),
                    }
                }
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

/// The x channel: a borrowed series, or generated indices `0, 1, 2, …`.
pub(crate) fn index_or_borrow<'p>(
    x: Option<&'p crate::data::Series<'_>>,
    len: usize,
) -> Cow<'p, [f64]> {
    match x {
        Some(series) => Cow::Borrowed(series.as_slice()),
        None => Cow::Owned((0..len).map(|i| i as f64).collect()),
    }
}

/// The finite `(min, max)` over strictly positive values, or `None` without any.
pub(crate) fn extent_positive(values: &[f64]) -> Option<(f64, f64)> {
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
pub(crate) fn extent(values: &[f64]) -> Option<(f64, f64)> {
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
pub(crate) fn union(extents: impl Iterator<Item = Option<(f64, f64)>>) -> Option<(f64, f64)> {
    extents
        .flatten()
        .reduce(|(min_a, max_a), (min_b, max_b)| (min_a.min(min_b), max_a.max(max_b)))
}
