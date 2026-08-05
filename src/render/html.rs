//! Small HTML helpers for the Evcxr cell-grid encoder.

use crate::Theme;

/// Escapes one text-cell glyph into HTML element content.
pub(super) fn escape(glyph: char, out: &mut String) {
    match glyph {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        glyph => out.push(glyph),
    }
}

/// The card background and foreground for a theme.
pub(crate) fn card(theme: Theme) -> (&'static str, &'static str) {
    if theme == Theme::LIGHT {
        ("#ffffff", "#1f2328")
    } else {
        ("#0d1117", "#e6edf3")
    }
}

/// Wraps mime-typed fragments in Evcxr's stdout protocol as alternative
/// representations. A frontend renders the richest it supports — so a `text/plain`
/// block beside `text/html` yields the card in Jupyter and a plain plot in the
/// terminal REPL, which cannot draw HTML.
pub(crate) fn mime_bundle(blocks: &[(&str, &str)]) -> String {
    use std::fmt::Write as _;

    let mut out = String::new();
    for (mime, content) in blocks {
        if !out.is_empty() {
            out.push('\n');
        }
        let _ = write!(
            out,
            "EVCXR_BEGIN_CONTENT {mime}\n{content}\nEVCXR_END_CONTENT"
        );
    }
    out
}

#[cfg(test)]
#[path = "tests/html_tests.rs"]
mod tests;
