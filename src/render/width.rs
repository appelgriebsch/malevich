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

/// Display width with ANSI/ECMA-48 escape strings treated as zero-width.
///
/// Besides the CSI color sequences emitted by the cell encoder, this recognizes
/// OSC and ST-terminated control strings so the scanner is safe to reuse at other
/// terminal composition boundaries.
pub(crate) fn display_width_ansi(text: &str) -> usize {
    let mut width = 0;
    let mut chars = text.chars().peekable();
    while let Some(glyph) = chars.next() {
        if glyph != '\u{1b}' {
            width += glyph.width().unwrap_or(0);
            continue;
        }
        match chars.next() {
            Some('[') => {
                // CSI ends at the first final byte in 0x40..=0x7e.
                for next in chars.by_ref() {
                    if next.is_ascii() && ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
            Some(']') => skip_control_string(&mut chars, true),
            Some('P' | '_' | '^' | 'X') => skip_control_string(&mut chars, false),
            Some(_) | None => {}
        }
    }
    width
}

fn skip_control_string(
    chars: &mut std::iter::Peekable<impl Iterator<Item = char>>,
    bell_terminated: bool,
) {
    while let Some(next) = chars.next() {
        if bell_terminated && next == '\u{7}' {
            return;
        }
        if next == '\u{1b}' && chars.next_if_eq(&'\\').is_some() {
            return;
        }
    }
}

/// Fits `text` into at most `max` columns, truncating with `ellipsis` when needed.
pub(crate) fn fit_width_with(text: &str, max: usize, ellipsis: char) -> String {
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
        out.push(ellipsis);
    }
    out
}

#[cfg(test)]
#[path = "tests/width_tests.rs"]
mod tests;
