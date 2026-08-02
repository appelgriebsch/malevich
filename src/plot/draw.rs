//! Mark drawing: resolved layers rasterized onto the surface through the layout.

use crate::mark::{Orientation, Placement};
use crate::plot::layout::{Layout, Map};
use crate::plot::resolve::{Kind, ResolvedLayer, extent};
use crate::render::{Charset, Color, Surface};
use crate::scale::Colormap;

/// Draws every resolved layer, in order, through the shared scales.
pub(crate) fn layers(surface: &mut Surface, layout: &Layout<'_>, layers: &[ResolvedLayer<'_>]) {
    let Layout {
        px,
        py,
        gutter,
        plot_top,
        plot_rows,
        plot_cols,
        plot_sub_w,
        plot_sub_h,
        x_offset,
        y_offset,
        ..
    } = *layout;
    let x_scale = &layout.x_scale;
    let y_scale = &layout.y_scale;
    let band = &layout.band;
    let charset = layout.charset;
    for layer in layers {
        match layer {
            ResolvedLayer::Series {
                x, y, color, kind, ..
            } => {
                draw_series(
                    surface,
                    kind,
                    x,
                    y,
                    *color,
                    x_scale,
                    y_scale,
                    (x_offset, y_offset),
                );
            }
            ResolvedLayer::Area {
                x,
                low,
                high,
                horizontal,
                color,
                ..
            } => {
                draw_area(
                    surface,
                    x,
                    *low,
                    high,
                    *horizontal,
                    *color,
                    x_scale,
                    y_scale,
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
                    surface,
                    *columns,
                    values,
                    *extents,
                    *colormap,
                    x_scale,
                    y_scale,
                    (gutter, plot_top, plot_cols, plot_rows),
                    (px, py),
                );
            }
            ResolvedLayer::Range {
                x,
                low,
                high,
                body,
                marker,
                color,
                ..
            } => {
                let half_width = match &band {
                    Some(band) => band.bandwidth() * 0.3,
                    None => px as f64,
                };
                draw_ranges(
                    surface,
                    x,
                    low,
                    high,
                    *body,
                    *marker,
                    *color,
                    x_scale,
                    y_scale,
                    (x_offset, y_offset),
                    (half_width, px as f64, py as f64),
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
                            surface,
                            &|index| {
                                (
                                    band.position(index),
                                    band.position(index) + band.bandwidth(),
                                )
                            },
                            y_scale,
                            values,
                            *color,
                            (gutter, plot_top, plot_rows),
                            (px, py),
                            charset,
                        );
                    }
                }
                Placement::Spans { start, width } => {
                    draw_bars(
                        surface,
                        &|index| {
                            let left = x_scale.map(start + width * index as f64);
                            let right = x_scale.map(start + width * (index + 1) as f64);
                            (left, right)
                        },
                        y_scale,
                        values,
                        *color,
                        (gutter, plot_top, plot_rows),
                        (px, py),
                        charset,
                    );
                }
            },
        }
    }
}

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

/// Draws one range layer: per interval, a thin capped whisker from `low` to
/// `high`, an optional thick body (filled with vertical subpixel runs, like areas),
/// and an optional marker crossbar written as text so it stays visible over the
/// fill in every charset.
#[allow(clippy::too_many_arguments)]
fn draw_ranges(
    surface: &mut Surface,
    x: &[f64],
    low: &[f64],
    high: &[f64],
    body: Option<(&[f64], &[f64])>,
    marker: Option<&[f64]>,
    color: Color,
    x_scale: &Map,
    y_scale: &Map,
    offset: (f64, f64),
    geometry: (f64, f64, f64),
) {
    let (half_width, px, py) = geometry;
    let cap = (half_width * 0.6).max(1.0);
    for index in 0..low.len() {
        let (xv, lv, hv) = (x[index], low[index], high[index]);
        if !xv.is_finite() || !lv.is_finite() || !hv.is_finite() {
            continue;
        }
        let sx = offset.0 + x_scale.map(xv);
        let sl = offset.1 + y_scale.map(lv);
        let sh = offset.1 + y_scale.map(hv);
        // The whisker and its caps.
        surface.line((sx, sl), (sx, sh), color);
        surface.line((sx - cap, sl), (sx + cap, sl), color);
        surface.line((sx - cap, sh), (sx + cap, sh), color);
        // The body: vertical subpixel runs across the width.
        if let Some((body_low, body_high)) = body {
            let (bl, bh) = (body_low[index], body_high[index]);
            if bl.is_finite() && bh.is_finite() {
                let sbl = offset.1 + y_scale.map(bl);
                let sbh = offset.1 + y_scale.map(bh);
                let from = (sx - half_width).round() as i64;
                let to = (sx + half_width).round() as i64;
                for column in from..=to {
                    surface.line((column as f64, sbl), (column as f64, sbh), color);
                }
            }
        }
        // The marker crossbar, as text: it must read over the fill.
        if let Some(marker) = marker {
            let mv = marker[index];
            if mv.is_finite() {
                let sy = offset.1 + y_scale.map(mv);
                let row = (sy / py).round() as i64;
                let from_cell = ((sx - half_width) / px).round() as i64;
                let to_cell = ((sx + half_width) / px).round() as i64;
                for cell in from_cell..=to_cell {
                    surface.text(cell, row, "\u{2501}", color);
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
    channel: &[f64],
    low: Option<&[f64]>,
    high: &[f64],
    horizontal: bool,
    color: Color,
    x_scale: &Map,
    y_scale: &Map,
    offset: (f64, f64),
) {
    // In the vertical (default) orientation the channel is x and fills run in y;
    // horizontally the channel is y and fills run in x. `place` restores raster
    // coordinates from (main, cross).
    let place = |main: f64, cross: f64| -> (f64, f64) {
        if horizontal {
            (cross, main)
        } else {
            (main, cross)
        }
    };
    let mut previous: Option<(f64, f64, f64)> = None;
    for index in 0..high.len() {
        let cv = channel[index];
        let hv = high[index];
        let lv = low.map_or(0.0, |low| low[index]);
        if !cv.is_finite() || !hv.is_finite() || !lv.is_finite() {
            previous = None;
            continue;
        }
        let (main, cross_low, cross_high) = if horizontal {
            (
                offset.1 + y_scale.map(cv),
                offset.0 + x_scale.map(lv),
                offset.0 + x_scale.map(hv),
            )
        } else {
            (
                offset.0 + x_scale.map(cv),
                offset.1 + y_scale.map(lv),
                offset.1 + y_scale.map(hv),
            )
        };
        match previous {
            Some((pm, pl, ph)) => {
                let (from, to) = if pm <= main { (pm, main) } else { (main, pm) };
                let span = main - pm;
                for step in (from.round() as i64)..=(to.round() as i64) {
                    let t = if span.abs() < f64::EPSILON {
                        0.0
                    } else {
                        ((step as f64 - pm) / span).clamp(0.0, 1.0)
                    };
                    let step_low = pl + (cross_low - pl) * t;
                    let step_high = ph + (cross_high - ph) * t;
                    surface.line(
                        place(step as f64, step_low),
                        place(step as f64, step_high),
                        color,
                    );
                }
            }
            None => surface.line(place(main, cross_low), place(main, cross_high), color),
        }
        previous = Some((main, cross_low, cross_high));
    }
}
