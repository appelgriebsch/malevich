//! The subpixel surface: the grid marks draw on, and its string encoders.

use super::canvas::{Canvas, PlotRect};
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
    /// An optional drawing clip in subpixel coordinates `(x0, y0, x1, y1)`, upper
    /// bounds exclusive. When set, [`Surface::set`] and [`Surface::text`] draw only
    /// inside it — used to confine marks to the plot rectangle so their ink never
    /// escapes into the axes or gutter. Chrome is drawn with no clip.
    clip: Option<(i64, i64, i64, i64)>,
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
            clip: None,
        }
    }

    /// Confines subsequent drawing to the subpixel rectangle `[x0, x1) x [y0, y1)`.
    pub(crate) fn set_clip(&mut self, x0: i64, y0: i64, x1: i64, y1: i64) {
        self.clip = Some((x0, y0, x1, y1));
    }

    /// Removes any drawing clip.
    pub(crate) fn clear_clip(&mut self) {
        self.clip = None;
    }

    /// The size in cells as `(width, height)`.
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// The size in subpixels as `(width, height)`.
    pub fn subpixel_size(&self) -> (usize, usize) {
        (self.width * self.columns, self.height * self.rows)
    }

    /// Sets the subpixel at `(x, y)`; outside the surface or the active clip this
    /// does nothing.
    pub fn set(&mut self, x: i64, y: i64, color: Color) {
        let (sw, sh) = self.subpixel_size();
        if x < 0 || y < 0 || x >= sw as i64 || y >= sh as i64 {
            return;
        }
        if let Some((x0, y0, x1, y1)) = self.clip
            && (x < x0 || y < y0 || x >= x1 || y >= y1)
        {
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
        // Clip to the surface, tightened to the active clip rectangle: this bounds
        // the Bresenham walk to the drawable region, so even wildly out-of-range
        // finite endpoints cost only the pixels actually on screen.
        let (mut wx0, mut wy0, mut wx1, mut wy1) =
            (0.0f64, 0.0f64, (sw - 1) as f64, (sh - 1) as f64);
        if let Some((x0, y0, x1, y1)) = self.clip {
            wx0 = wx0.max(x0 as f64);
            wy0 = wy0.max(y0 as f64);
            wx1 = wx1.min((x1 - 1) as f64);
            wy1 = wy1.min((y1 - 1) as f64);
        }
        super::canvas::trace_line(from, to, (wx0, wy0, wx1, wy1), |x, y| {
            self.set(x, y, color);
        });
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
        if let Some((x0, y0, x1, y1)) = self.clip {
            let (col, row) = (column as i64, row as i64);
            let (px, py) = (self.columns as i64, self.rows as i64);
            if col < x0 / px || col >= x1 / px || row < y0 / py || row >= y1 / py {
                return;
            }
        }
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

    /// Encodes the cell grid as HTML element content with concrete-RGB span runs.
    ///
    /// Default-colored glyphs inherit from their enclosing element. Rows are
    /// newline-joined with trailing spaces trimmed, just like [`Surface::encode`].
    #[cfg(feature = "evcxr")]
    pub(crate) fn encode_html(&self) -> String {
        use std::fmt::Write as _;

        let mut out = String::with_capacity((self.width + 32) * self.height);
        for row in 0..self.height {
            if row > 0 {
                out.push('\n');
            }
            let mut current = None;
            let mut kept = out.len();
            for (glyph, color) in self.row(row) {
                // Spaces carry no visible color; letting them inherit the current
                // one lengthens runs and keeps trailing whitespace trimmable.
                if glyph != ' ' {
                    let next = match color {
                        Color::Default => None,
                        color => Some(color.to_rgb()),
                    };
                    if next != current {
                        if current.is_some() {
                            out.push_str("</span>");
                        }
                        if let Some((r, g, b)) = next {
                            let _ = write!(out, "<span style=\"color:#{r:02x}{g:02x}{b:02x}\">");
                        }
                        current = next;
                    }
                }
                super::html::escape(glyph, &mut out);
                if glyph != ' ' {
                    kept = out.len();
                }
            }
            out.truncate(kept);
            if current.is_some() {
                out.push_str("</span>");
            }
        }
        out
    }

    /// Every printable cell as `(column, row, glyph, color)`, skipping wide-glyph
    /// continuations (the glyph to their left covers them). For adapters that write
    /// into cell buffers instead of strings.
    #[cfg_attr(not(feature = "ratatui"), allow(dead_code))]
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

impl Canvas for Surface {
    fn set_clip(&mut self, x0: i64, y0: i64, x1: i64, y1: i64) {
        Surface::set_clip(self, x0, y0, x1, y1);
    }

    fn clear_clip(&mut self) {
        Surface::clear_clip(self);
    }

    fn dot(&mut self, x: f64, y: f64, color: Color) {
        Surface::dot(self, x, y, color);
    }

    fn line(&mut self, from: (f64, f64), to: (f64, f64), color: Color) {
        Surface::line(self, from, to, color);
    }

    fn text(&mut self, column: i64, row: i64, text: &str, color: Color) {
        Surface::text(self, column, row, text, color);
    }

    /// One bar as cell-aligned columns from the zero baseline, with eighth-block
    /// partial fills at the value end (upward bars) or coarse upper-block fills
    /// (downward bars — Unicode has no lower-anchored upper ramp).
    fn bar(
        &mut self,
        span: (f64, f64),
        end: f64,
        baseline: f64,
        positive: bool,
        rect: PlotRect,
        color: Color,
    ) {
        let (px, py) = (self.columns, self.rows);
        let ramp = self.charset.fill_ramp();
        let eighths = ramp.len() == 8;
        let mut buffer = [0u8; 4];
        let baseline = baseline / py as f64;
        let end = end / py as f64;
        let left = (span.0 / px as f64).round() as i64;
        let right = ((span.1 / px as f64).round() as i64).max(left + 1);
        // Clamp to the plot columns before iterating: a bar whose span maps far
        // off-screen (distant data under a narrow domain) must not spin a giant
        // loop just to have every cell clipped away.
        let left = left.clamp(0, rect.columns as i64);
        let right = right.clamp(0, rect.columns as i64);

        for column in left..right {
            let cell_column = rect.gutter as i64 + column;
            if positive {
                // Upward: full cells from the (snapped-down) baseline, a
                // bottom-anchored partial at the top.
                let bottom = baseline.ceil().min(rect.rows as f64);
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
                        Surface::text(
                            self,
                            cell_column,
                            rect.top as i64 + row as i64,
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
                let bottom = end.min(rect.rows as f64);
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
                        Surface::text(
                            self,
                            cell_column,
                            rect.top as i64 + row as i64,
                            glyph.encode_utf8(&mut buffer),
                            color,
                        );
                    }
                    row += 1.0;
                }
            }
        }
    }

    /// The marker crossbar as text: a run of the chrome marker glyph, which reads
    /// over a same-color fill because the glyph differs from the fill texture.
    fn marker(&mut self, sx: f64, half_width: f64, sy: f64, color: Color) {
        let (px, py) = (self.columns as f64, self.rows as f64);
        let row = (sy / py).round() as i64;
        let from_cell = ((sx - half_width) / px).round() as i64;
        let to_cell = ((sx + half_width) / px).round() as i64;
        let glyph = self.charset.chrome().marker;
        for cell in from_cell..=to_cell {
            Surface::text(self, cell, row, glyph, color);
        }
    }

    fn patch_size(&self) -> (usize, usize) {
        (self.columns, self.rows)
    }

    /// One Cells patch as a shade-ramp glyph colored by the colormap — value in
    /// glyph and color both, readable at every color tier.
    fn patch(&mut self, column: usize, row: usize, rect: PlotRect, intensity: f64, color: Color) {
        const RAMP: [char; 4] = ['\u{2591}', '\u{2592}', '\u{2593}', '\u{2588}'];
        let mut buffer = [0u8; 4];
        let glyph = RAMP[((intensity * 4.0) as usize).min(3)];
        Surface::text(
            self,
            (rect.gutter + column) as i64,
            (rect.top + row) as i64,
            glyph.encode_utf8(&mut buffer),
            color,
        );
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

#[cfg(test)]
#[path = "tests/surface_tests.rs"]
mod tests;
