//! The subpixel surface: the grid marks draw on, and its string encoders.

use super::charset::Charset;
use super::color::{Color, ColorMode, Resolved};

/// One character cell: a subpixel pattern, a text slot, and a color.
///
/// Text wins over pixels when the cell prints — labels are never corrupted by marks
/// drawing underneath them.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Cell {
    bits: u8,
    text: Text,
    color: Color,
}

/// The text slot of a cell. A wide glyph (CJK) occupies its own cell plus a
/// `Continuation` to its right; the pair is kept consistent on every overwrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Text {
    None,
    Glyph(char),
    Continuation,
}

const EMPTY: Cell = Cell {
    bits: 0,
    text: Text::None,
    color: Color::Default,
};

/// A grid of character cells addressed in subpixel coordinates.
///
/// The surface is pure raster state: origin at the top-left, y growing downward,
/// `width * height` cells at the charset's subpixel density. Drawing is infallible —
/// coordinates outside the surface clip away, non-finite coordinates draw nothing.
/// When several colors land in one cell, the last write wins.
#[derive(Clone, PartialEq)]
pub struct Surface {
    width: usize,
    height: usize,
    charset: Charset,
    columns: usize,
    rows: usize,
    cells: Vec<Cell>,
}

impl Surface {
    /// Creates an empty surface of `width * height` cells encoded with `charset`.
    pub fn new(width: usize, height: usize, charset: Charset) -> Surface {
        let (columns, rows) = charset.pixels_per_cell();
        Surface {
            width,
            height,
            charset,
            columns,
            rows,
            cells: vec![EMPTY; width * height],
        }
    }

    /// The size in cells as `(width, height)`.
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// The size in subpixels as `(width, height)`.
    pub fn subpixel_size(&self) -> (usize, usize) {
        (self.width * self.columns, self.height * self.rows)
    }

    /// Sets the subpixel at `(x, y)`; outside the surface this does nothing.
    pub fn set(&mut self, x: i64, y: i64, color: Color) {
        let (sw, sh) = self.subpixel_size();
        if x < 0 || y < 0 || x >= sw as i64 || y >= sh as i64 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        let index = (y / self.rows) * self.width + x / self.columns;
        let cell = &mut self.cells[index];
        cell.bits |= self.charset.bit(x % self.columns, y % self.rows);
        cell.color = color;
    }

    /// Sets the subpixel nearest to `(x, y)`; non-finite coordinates draw nothing.
    pub fn dot(&mut self, x: f64, y: f64, color: Color) {
        if x.is_finite() && y.is_finite() {
            self.set(x.round() as i64, y.round() as i64, color);
        }
    }

    /// Draws a line between two subpixel positions, clipped to the surface.
    ///
    /// Non-finite endpoints draw nothing (a gap breaks a polyline before it reaches
    /// this call, but the surface defends itself regardless).
    pub fn line(&mut self, from: (f64, f64), to: (f64, f64), color: Color) {
        if !(from.0.is_finite() && from.1.is_finite() && to.0.is_finite() && to.1.is_finite()) {
            return;
        }
        let (sw, sh) = self.subpixel_size();
        if sw == 0 || sh == 0 {
            return;
        }
        let window = ((sw - 1) as f64, (sh - 1) as f64);
        let Some((from, to)) = clip(from, to, window) else {
            return;
        };
        let (x1, y1) = (to.0.round() as i64, to.1.round() as i64);
        let (mut x, mut y) = (from.0.round() as i64, from.1.round() as i64);
        let dx = (x1 - x).abs();
        let dy = -(y1 - y).abs();
        let sx = if x < x1 { 1 } else { -1 };
        let sy = if y < y1 { 1 } else { -1 };
        let mut error = dx + dy;
        loop {
            self.set(x, y, color);
            if x == x1 && y == y1 {
                return;
            }
            let doubled = 2 * error;
            if doubled >= dy {
                error += dy;
                x += sx;
            }
            if doubled <= dx {
                error += dx;
                y += sy;
            }
        }
    }

    /// Writes text starting at the cell `(column, row)`; cells outside clip away.
    ///
    /// Text overrides any pixels in the same cells and is measured in display
    /// columns: a wide glyph (CJK) occupies two cells, and one that would straddle
    /// the surface edge is dropped whole. Zero-width characters (combining marks) do
    /// not survive the cell grid and are dropped. Overwriting half of a wide glyph
    /// blanks its other half — alignment is never corrupted.
    pub fn text(&mut self, column: i64, row: i64, text: &str, color: Color) {
        use unicode_width::UnicodeWidthChar;

        if row < 0 || row >= self.height as i64 {
            return;
        }
        let row = row as usize;
        let mut column = column;
        for glyph in text.chars() {
            let width = glyph.width().unwrap_or(0) as i64;
            if width == 0 {
                continue;
            }
            let fits = column >= 0 && column + width <= self.width as i64;
            if fits {
                self.place(row, column as usize, Text::Glyph(glyph), color);
                for offset in 1..width {
                    self.place(row, (column + offset) as usize, Text::Continuation, color);
                }
            }
            column += width;
        }
    }

    /// Puts one text slot into a cell, breaking any wide-glyph pair it overlaps.
    fn place(&mut self, row: usize, column: usize, text: Text, color: Color) {
        let base = row * self.width;
        // Overwriting a continuation orphans the wide glyph to its left.
        if self.cells[base + column].text == Text::Continuation && column > 0 {
            self.cells[base + column - 1].text = Text::Glyph(' ');
        }
        // Overwriting a wide glyph orphans its continuation to the right.
        if column + 1 < self.width && self.cells[base + column + 1].text == Text::Continuation {
            self.cells[base + column + 1].text = Text::Glyph(' ');
        }
        let cell = &mut self.cells[base + column];
        cell.text = text;
        cell.color = color;
    }

    /// Encodes as plain text — no escape codes ever. Sugar for
    /// [`Surface::encode`] with [`ColorMode::Plain`].
    pub fn to_plain(&self) -> String {
        self.encode(ColorMode::Plain)
    }

    /// Encodes the surface at the color tier of `mode`.
    ///
    /// Colors resolve to what the mode can carry (RGB quantizes downhill; see
    /// [`Color`]); an SGR sequence is emitted only when the resolved color changes
    /// along a row, so colors that quantize identically share one sequence, and any
    /// colored row ends with a reset. Rows are joined by newlines with trailing
    /// spaces trimmed. In [`ColorMode::Plain`] the output carries no escapes at all.
    pub fn encode(&self, mode: ColorMode) -> String {
        let mut out = String::with_capacity((self.width + 8) * self.height);
        for row in 0..self.height {
            if row > 0 {
                out.push('\n');
            }
            let mut current = Resolved::Default;
            let mut kept = out.len();
            for (glyph, color) in self.row(row) {
                // Spaces carry no visible color; letting them inherit the current
                // one lengthens runs and keeps trailing whitespace trimmable.
                if glyph != ' ' {
                    let resolved = color.resolve(mode);
                    if resolved != current {
                        resolved.write_sgr(&mut out);
                        current = resolved;
                    }
                }
                out.push(glyph);
                if glyph != ' ' {
                    kept = out.len();
                }
            }
            out.truncate(kept);
            if current != Resolved::Default {
                out.push_str("\x1b[0m");
            }
        }
        out
    }

    /// Every printable cell as `(column, row, glyph, color)`, skipping wide-glyph
    /// continuations (the glyph to their left covers them). For adapters that write
    /// into cell buffers instead of strings.
    pub(crate) fn cells(&self) -> impl Iterator<Item = (usize, usize, char, Color)> + '_ {
        self.cells.iter().enumerate().filter_map(|(index, cell)| {
            let (row, column) = (index / self.width.max(1), index % self.width.max(1));
            match cell.text {
                Text::Continuation => None,
                Text::Glyph(glyph) => Some((column, row, glyph, cell.color)),
                Text::None => Some((column, row, self.charset.glyph(cell.bits), cell.color)),
            }
        })
    }

    /// The printable glyphs of one row, in order. Continuation cells emit nothing:
    /// the wide glyph to their left covers their column.
    fn row(&self, row: usize) -> impl Iterator<Item = (char, Color)> + '_ {
        self.cells[row * self.width..(row + 1) * self.width]
            .iter()
            .filter_map(|cell| match cell.text {
                Text::Continuation => None,
                Text::Glyph(glyph) => Some((glyph, cell.color)),
                Text::None => Some((self.charset.glyph(cell.bits), cell.color)),
            })
    }
}

impl std::fmt::Debug for Surface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Surface")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("charset", &self.charset)
            .finish_non_exhaustive()
    }
}

/// Liang–Barsky clipping of the segment `from → to` against `[0, window.0] x
/// [0, window.1]`; `None` when the segment lies entirely outside.
fn clip(from: (f64, f64), to: (f64, f64), window: (f64, f64)) -> Option<((f64, f64), (f64, f64))> {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let mut enter = 0.0f64;
    let mut exit = 1.0f64;
    let tests = [
        (-dx, from.0),
        (dx, window.0 - from.0),
        (-dy, from.1),
        (dy, window.1 - from.1),
    ];
    for (p, q) in tests {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                if r > exit {
                    return None;
                }
                enter = enter.max(r);
            } else {
                if r < enter {
                    return None;
                }
                exit = exit.min(r);
            }
        }
    }
    if enter > exit {
        return None;
    }
    Some((
        (from.0 + enter * dx, from.1 + enter * dy),
        (from.0 + exit * dx, from.1 + exit * dy),
    ))
}

#[cfg(test)]
#[path = "tests/surface_tests.rs"]
mod tests;
