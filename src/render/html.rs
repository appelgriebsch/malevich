//! The HTML escaper for the Evcxr cell-grid encoder.
//!
//! The card colors and the stdout protocol live in [`crate::evcxr`],
//! which is public: a crate rendering its own types beside a malevich
//! chart needs both, and they are useless if only malevich can reach them.

/// Escapes one text-cell glyph into HTML element content.
pub(super) fn escape(glyph: char, out: &mut String) {
    match glyph {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        glyph => out.push(glyph),
    }
}

#[cfg(test)]
#[path = "tests/html_tests.rs"]
mod tests;
