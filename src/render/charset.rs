//! Charset codecs: how a cell's subpixel pattern becomes a glyph.
//!
//! Glyph selection is data (bit masks and offsets), not code: adding a charset is a
//! table. Each codec defines its subpixel density and the mapping from a subpixel
//! position to a bit in the cell's pattern.

/// A glyph tier for encoding the surface.
///
/// Richer tiers draw the same subpixels with better glyphs; the choice never affects
/// what marks draw, only how cells print.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    /// Pure ASCII: one pixel per cell, drawn as `*`. Works on any terminal.
    Ascii,
    /// Braille patterns (U+2800–U+28FF): 2×4 pixels per cell. Universally available
    /// in monospace fonts; the default for lines and scatter.
    Braille,
}

/// Braille dot masks in row-major subpixel order: index `row * 2 + column`.
///
/// Unicode assigns dots 1–3 and 7 to the left column (top to bottom) and dots 4–6
/// and 8 to the right column; this table hides that historical layout.
const BRAILLE_DOTS: [u8; 8] = [0x01, 0x08, 0x02, 0x10, 0x04, 0x20, 0x40, 0x80];

impl Charset {
    /// Subpixels per cell as `(columns, rows)`.
    pub fn pixels_per_cell(self) -> (usize, usize) {
        match self {
            Charset::Ascii => (1, 1),
            Charset::Braille => (2, 4),
        }
    }

    /// The pattern bit for the subpixel at `(column, row)` within a cell.
    pub(crate) fn bit(self, column: usize, row: usize) -> u8 {
        match self {
            Charset::Ascii => 1,
            Charset::Braille => BRAILLE_DOTS[row * 2 + column],
        }
    }

    /// The glyph for a cell's pattern; an empty pattern is a space.
    pub(crate) fn glyph(self, bits: u8) -> char {
        if bits == 0 {
            return ' ';
        }
        match self {
            Charset::Ascii => '*',
            Charset::Braille => {
                char::from_u32(0x2800 + u32::from(bits)).expect("braille block is contiguous")
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/charset_tests.rs"]
mod tests;
