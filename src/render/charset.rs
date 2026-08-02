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
    /// Half blocks (`▀▄█`): 1×2 pixels per cell. Solid, ancient, everywhere.
    HalfBlocks,
    /// Quadrants (`▘▚▟`…): 2×2 pixels per cell. Solid blocks at four pixels a cell.
    Quadrants,
    /// Braille patterns (U+2800–U+28FF): 2×4 pixels per cell. Universally available
    /// in monospace fonts; the default for lines and scatter.
    Braille,
}

/// Quadrant glyphs indexed by bit pattern: bit 0 top-left, bit 1 top-right,
/// bit 2 bottom-left, bit 3 bottom-right.
const QUADRANTS: [char; 16] = [
    ' ', '\u{2598}', '\u{259D}', '\u{2580}', '\u{2596}', '\u{258C}', '\u{259E}', '\u{259B}',
    '\u{2597}', '\u{259A}', '\u{2590}', '\u{259C}', '\u{2584}', '\u{2599}', '\u{259F}', '\u{2588}',
];

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
            Charset::HalfBlocks => (1, 2),
            Charset::Quadrants => (2, 2),
            Charset::Braille => (2, 4),
        }
    }

    /// The pattern bit for the subpixel at `(column, row)` within a cell.
    pub(crate) fn bit(self, column: usize, row: usize) -> u8 {
        match self {
            Charset::Ascii => 1,
            Charset::HalfBlocks => 1 << row,
            Charset::Quadrants => 1 << (row * 2 + column),
            Charset::Braille => BRAILLE_DOTS[row * 2 + column],
        }
    }

    /// The bottom-anchored fill ramp used by columnar marks: `ramp[k]` covers
    /// `(k + 1) / len` of a cell from the bottom up. ASCII has a single full-cell
    /// glyph; everything richer gets the eight eighth-blocks.
    pub(crate) fn fill_ramp(self) -> &'static [char] {
        match self {
            Charset::Ascii => &['#'],
            _ => &[
                '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
                '\u{2588}',
            ],
        }
    }

    /// The glyph for a cell's pattern; an empty pattern is a space.
    pub(crate) fn glyph(self, bits: u8) -> char {
        if bits == 0 {
            return ' ';
        }
        match self {
            Charset::Ascii => '*',
            Charset::HalfBlocks => match bits {
                1 => '\u{2580}',
                2 => '\u{2584}',
                _ => '\u{2588}',
            },
            Charset::Quadrants => QUADRANTS[usize::from(bits & 0x0F)],
            Charset::Braille => {
                char::from_u32(0x2800 + u32::from(bits)).expect("braille block is contiguous")
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/charset_tests.rs"]
mod tests;
