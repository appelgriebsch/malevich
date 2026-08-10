//! Evcxr notebook output: the stdout protocol, and the card colors.
//!
//! [`Plot::evcxr_display`](crate::Plot::evcxr_display) already draws a
//! plot in a notebook cell, and most callers need nothing else. This
//! module is for the case where a crate renders its *own* types
//! alongside malevich charts and wants the two to agree: emitting a
//! value through the same protocol, on the same background.
//!
//! ```
//! use malevich::evcxr::{card_colors, mime_bundle};
//! use malevich::Theme;
//!
//! let (background, foreground) = card_colors(Theme::DARK);
//! let html = format!(
//!     "<pre style=\"background-color:{background};color:{foreground}\">rows: 3</pre>"
//! );
//! println!("{}", mime_bundle(&[("text/html", &html), ("text/plain", "rows: 3")]));
//! ```
//!
//! Nothing here touches a terminal or reads the environment: both
//! functions are pure, so output built on them stays snapshot-testable.

use crate::Theme;

/// Returns the background and foreground a card uses for `theme`, as
/// CSS color literals.
///
/// These are the exact colors [`Plot::to_html`](crate::Plot::to_html)
/// paints its own card with. A crate rendering its own HTML beside a
/// malevich chart should draw from this rather than hardcode a pair,
/// so one notebook cell does not show two different backgrounds.
///
/// Only [`Theme::LIGHT`] selects the light card; every other theme,
/// including a custom palette, takes the dark one, which is the safer
/// default against an unknown notebook background.
///
/// # Examples
/// ```
/// use malevich::evcxr::card_colors;
/// use malevich::Theme;
///
/// assert_eq!(card_colors(Theme::LIGHT), ("#ffffff", "#1f2328"));
/// assert_eq!(card_colors(Theme::DARK), ("#0d1117", "#e6edf3"));
/// ```
pub fn card_colors(theme: Theme) -> (&'static str, &'static str) {
    if theme == Theme::LIGHT {
        ("#ffffff", "#1f2328")
    } else {
        ("#0d1117", "#e6edf3")
    }
}

/// Wraps mime-typed fragments in Evcxr's stdout protocol as
/// alternative representations of one value.
///
/// Printing the result from a type's `evcxr_display` is the whole
/// integration: Evcxr reads the blocks off stdout and the frontend
/// draws the richest form it supports. Pairing `text/plain` with
/// `text/html` is the usual choice — Jupyter takes the HTML, and the
/// terminal REPL, which cannot draw it, still shows something useful.
///
/// Blocks are emitted in the order given; content is passed through
/// untouched, so callers escape their own text.
///
/// # Examples
/// ```
/// use malevich::evcxr::mime_bundle;
///
/// assert_eq!(
///     mime_bundle(&[("text/plain", "3")]),
///     "EVCXR_BEGIN_CONTENT text/plain\n3\nEVCXR_END_CONTENT"
/// );
/// ```
pub fn mime_bundle(blocks: &[(&str, &str)]) -> String {
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
#[path = "evcxr_tests.rs"]
mod tests;
