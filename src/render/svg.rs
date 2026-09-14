//! The SVG card: the cell grid as an image any SVG host can draw.
//!
//! A page that draws with SVG is one more terminal. The card is a picture of
//! the raster, nothing more: glyphs that *are* block patterns — Block
//! Elements, sextants, octants — become the rectangles they denote, crisp at
//! any size; everything else (braille, box drawing, labels, titles) is a
//! `<text>` run the host's own font draws, pinned to the cell grid with
//! `textLength` so the layout survives whatever font that is. Colors resolve
//! exactly as the HTML card resolves them. No font is rasterized and no mark is
//! drawn from its data; a chart that the terminal cannot show, the SVG cannot
//! show either.

use super::charset::{OCTANTS, QUADRANTS};
use super::color::Color;
use super::html::escape;
use super::raster::{Raster, RasterCell};
use crate::Theme;

/// Cell advance: the natural 13px advance of the common monospace fonts, so a
/// renderer that ignores `textLength` still drifts by a fraction of a cell.
const CELL_W: f64 = 7.8;
/// Row pitch.
const CELL_H: f64 = 16.0;
const FONT_PX: f64 = 13.0;
/// Text baseline below the row's top edge.
const BASELINE: f64 = 12.5;
/// The HTML card's padding and corner radius, so the two cards match.
const PAD_X: f64 = 16.0;
const PAD_Y: f64 = 12.0;
const RADIUS: f64 = 8.0;
const FONT_FAMILY: &str = "ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace";

/// The filled sub-cells of a block glyph on a `columns × rows` grid, one bit per
/// sub-cell in row-major order (top-left first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ink {
    columns: u8,
    rows: u8,
    bits: u8,
}

impl Ink {
    fn bit(self, column: u8, row: u8) -> bool {
        self.bits & (1 << (row * self.columns + column)) != 0
    }

    /// The bits of one column, as a row mask.
    fn column(self, column: u8) -> u8 {
        (0..self.rows)
            .filter(|&row| self.bit(column, row))
            .fold(0, |mask, row| mask | (1 << row))
    }

    /// Whether every ink rectangle spans the full cell width, so runs of this
    /// glyph merge into one rectangle per row band.
    fn full_width(self) -> bool {
        (1..self.columns).all(|column| self.column(column) == self.column(0))
    }
}

/// The ink of a glyph that is defined as a block pattern, or `None` for a glyph
/// the host's font should draw.
fn ink(glyph: char) -> Option<Ink> {
    if glyph == ' ' {
        return None;
    }
    if let Some(bits) = QUADRANTS.iter().position(|&candidate| candidate == glyph) {
        return Some(Ink {
            columns: 2,
            rows: 2,
            bits: bits as u8,
        });
    }
    let code = glyph as u32;
    Some(match code {
        // Lower eighths ▁..▇ (U+2581..U+2587): the bottom k of eight rows.
        0x2581..=0x2587 => {
            let k = code - 0x2580;
            Ink {
                columns: 1,
                rows: 8,
                bits: (((1u32 << k) - 1) << (8 - k)) as u8,
            }
        }
        // Left eighths ▉..▏ (U+2589..U+258F): the left k of eight columns.
        0x2589..=0x258F => {
            let k = 0x2590 - code;
            Ink {
                columns: 8,
                rows: 1,
                bits: ((1u32 << k) - 1) as u8,
            }
        }
        // Upper eighth ▔ and right eighth ▕.
        0x2594 => Ink {
            columns: 1,
            rows: 8,
            bits: 1,
        },
        0x2595 => Ink {
            columns: 8,
            rows: 1,
            bits: 1 << 7,
        },
        // Sextants: the codec's mapping, inverted (three patterns are legacy
        // glyphs the quadrant table already answered).
        0x1FB00..=0x1FB3B => {
            let index = code - 0x1FB00;
            let bits = index + 1 + u32::from(index >= 20) + u32::from(index >= 40);
            Ink {
                columns: 2,
                rows: 3,
                bits: bits as u8,
            }
        }
        // Octants (Unicode 16) and the legacy-computing glyphs the table reuses.
        0x1CC00..=0x1CEBF | 0x1FB80..=0x1FBEF => {
            let bits = OCTANTS.iter().position(|&candidate| candidate == glyph)?;
            Ink {
                columns: 2,
                rows: 4,
                bits: bits as u8,
            }
        }
        _ => return None,
    })
}

/// A number formatted with up to three decimals and no trailing zeros.
fn num(value: f64) -> String {
    let mut text = format!("{value:.3}");
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    text
}

fn hex(color: Color) -> String {
    let (r, g, b) = color.to_rgb();
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn rect(out: &mut String, x: f64, y: f64, width: f64, height: f64, fill: &str) {
    use std::fmt::Write as _;

    let _ = writeln!(
        out,
        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{fill}\"/>",
        num(x),
        num(y),
        num(width),
        num(height)
    );
}

impl Raster {
    /// Encodes the raster as a self-contained SVG terminal card.
    ///
    /// The sibling of [`Raster::to_html`] for hosts that draw with SVG — a
    /// README on GitHub, a notebook export, a static page. Block glyphs
    /// (Block Elements, sextants, octants) become crisp rectangles; every other
    /// glyph is a text run the host's font draws, pinned to the cell grid.
    /// Mark colors resolve to concrete RGB as in the HTML card; default-colored
    /// chrome takes the card foreground, and the card's background and
    /// foreground follow `theme` ([`Theme::LIGHT`] selects the light card).
    /// A plot's [`crate::Plot::to_svg`] is this encoding of its raster. Pure
    /// and deterministic.
    pub fn to_svg(&self, theme: Theme) -> String {
        use std::fmt::Write as _;

        let (columns, rows) = (self.width(), self.height());
        let width = 2.0 * PAD_X + columns as f64 * CELL_W;
        let height = 2.0 * PAD_Y + rows as f64 * CELL_H;
        let (background, foreground) = theme.card_colors();
        let mut out = String::with_capacity(512 + columns * rows * 8);
        let _ = writeln!(
            out,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\" font-family=\"{FONT_FAMILY}\" font-size=\"{FONT_PX}\" fill=\"{foreground}\" shape-rendering=\"crispEdges\">",
            w = num(width),
            h = num(height),
        );
        let _ = writeln!(
            out,
            "<rect width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{background}\"/>",
            num(width),
            num(height),
            num(RADIUS)
        );
        for row in 0..rows {
            let y = PAD_Y + row as f64 * CELL_H;
            let cells = &self.cells()[row * columns..(row + 1) * columns];
            self.svg_backgrounds(&mut out, cells, y);
            self.svg_ink(&mut out, cells, y, foreground);
            self.svg_text(&mut out, cells, y);
        }
        out.push_str("</svg>\n");
        out
    }

    /// Cell backgrounds, runs of one color merged; continuation cells extend
    /// the wide glyph's run.
    fn svg_backgrounds(&self, out: &mut String, cells: &[RasterCell], y: f64) {
        let mut column = 0;
        while column < cells.len() {
            let color = cells[column].background;
            if color == Color::Default {
                column += 1;
                continue;
            }
            let start = column;
            while column < cells.len() && cells[column].background == color {
                column += 1;
            }
            rect(
                out,
                PAD_X + start as f64 * CELL_W,
                y,
                (column - start) as f64 * CELL_W,
                CELL_H,
                &hex(color),
            );
        }
    }

    /// Block glyphs as rectangles: each contiguous run of set sub-cells in a
    /// grid column is one rectangle, identical adjacent columns share it, and
    /// runs of one full-width glyph in one color share every rectangle.
    fn svg_ink(&self, out: &mut String, cells: &[RasterCell], y: f64, foreground: &str) {
        let mut column = 0;
        while column < cells.len() {
            let cell = cells[column];
            let Some(ink) = (cell.columns > 0).then(|| ink(cell.glyph)).flatten() else {
                column += 1;
                continue;
            };
            let mut run = 1;
            if ink.full_width() {
                while column + run < cells.len()
                    && cells[column + run].glyph == cell.glyph
                    && cells[column + run].foreground == cell.foreground
                {
                    run += 1;
                }
            }
            let fill = match cell.foreground {
                Color::Default => foreground.to_string(),
                color => hex(color),
            };
            let fill = fill.as_str();
            let x0 = PAD_X + column as f64 * CELL_W;
            let sub_w = CELL_W / f64::from(ink.columns);
            let sub_h = CELL_H / f64::from(ink.rows);
            let mut grid_column = 0u8;
            while grid_column < ink.columns {
                let mask = ink.column(grid_column);
                let mut span = 1u8;
                while grid_column + span < ink.columns && ink.column(grid_column + span) == mask {
                    span += 1;
                }
                let mut grid_row = 0u8;
                while grid_row < ink.rows {
                    if mask & (1 << grid_row) == 0 {
                        grid_row += 1;
                        continue;
                    }
                    let from = grid_row;
                    while grid_row < ink.rows && mask & (1 << grid_row) != 0 {
                        grid_row += 1;
                    }
                    let x = x0 + f64::from(grid_column) * sub_w;
                    let width = if ink.full_width() {
                        run as f64 * CELL_W
                    } else {
                        f64::from(span) * sub_w
                    };
                    rect(
                        out,
                        x,
                        y + f64::from(from) * sub_h,
                        width,
                        f64::from(grid_row - from) * sub_h,
                        fill,
                    );
                }
                grid_column += span;
            }
            column += run;
        }
    }

    /// Text runs: consecutive non-block glyphs sharing a foreground, interior
    /// spaces kept so a label stays one element, wide glyphs counting two
    /// columns in the pinned length.
    fn svg_text(&self, out: &mut String, cells: &[RasterCell], y: f64) {
        use std::fmt::Write as _;

        let is_text =
            |cell: &RasterCell| cell.columns > 0 && cell.glyph != ' ' && ink(cell.glyph).is_none();
        let mut column = 0;
        while column < cells.len() {
            if !is_text(&cells[column]) {
                column += 1;
                continue;
            }
            let start = column;
            let color = cells[start].foreground;
            let mut text = String::new();
            let mut span = 0usize;
            let mut kept = (0usize, 0usize);
            while column < cells.len() {
                let cell = &cells[column];
                if cell.columns == 0 {
                    column += 1;
                    continue;
                }
                if cell.glyph != ' ' && (cell.foreground != color || ink(cell.glyph).is_some()) {
                    break;
                }
                escape(cell.glyph, &mut text);
                span += usize::from(cell.columns);
                if cell.glyph != ' ' {
                    kept = (text.len(), span);
                }
                column += 1;
            }
            text.truncate(kept.0);
            let fill = match color {
                Color::Default => String::new(),
                color => format!(" fill=\"{}\"", hex(color)),
            };
            let _ = writeln!(
                out,
                "<text x=\"{}\" y=\"{}\"{fill} textLength=\"{}\" lengthAdjust=\"spacingAndGlyphs\" xml:space=\"preserve\">{text}</text>",
                num(PAD_X + start as f64 * CELL_W),
                num(y + BASELINE),
                num(kept.1 as f64 * CELL_W),
            );
        }
    }
}

#[cfg(test)]
#[path = "tests/svg_tests.rs"]
mod tests;
