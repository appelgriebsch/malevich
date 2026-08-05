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

/// Wraps one HTML fragment in Evcxr's stdout MIME-block protocol.
pub(crate) fn mime_bundle(html: &str) -> String {
    format!("EVCXR_BEGIN_CONTENT text/html\n{html}\nEVCXR_END_CONTENT")
}

#[cfg(test)]
#[path = "tests/html_tests.rs"]
mod tests;
