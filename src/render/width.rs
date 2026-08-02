//! Display-width measurement and fitting for text on the cell grid.
//!
//! Labels are measured in terminal columns, not chars: CJK and other wide glyphs
//! occupy two cells. Zero-width characters (combining marks) do not survive the cell
//! grid — one cell holds one glyph — and are dropped at the boundary.

use unicode_width::UnicodeWidthChar;

/// The display width of `text` in terminal columns.
pub(crate) fn display_width(text: &str) -> usize {
    text.chars().map(|glyph| glyph.width().unwrap_or(0)).sum()
}

/// Fits `text` into at most `max` columns, truncating with `…` when needed.
pub(crate) fn fit_width(text: &str, max: usize) -> String {
    if display_width(text) <= max {
        return text.to_string();
    }
    let mut out = String::new();
    let mut used = 0;
    for glyph in text.chars() {
        let width = glyph.width().unwrap_or(0);
        if used + width > max.saturating_sub(1) {
            break;
        }
        out.push(glyph);
        used += width;
    }
    if max >= 1 {
        out.push('\u{2026}');
    }
    out
}

#[cfg(test)]
#[path = "tests/width_tests.rs"]
mod tests;
