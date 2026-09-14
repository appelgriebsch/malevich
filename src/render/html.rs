//! The HTML card: the cell grid as a `<pre>` of colored spans.
//!
//! A notebook is a terminal that draws with HTML. The card is the exact grid a
//! tty would print — glyphs, box drawing, braille — with mark colors as
//! concrete-RGB spans and chrome inheriting the card foreground, so the
//! browser draws the text the way a terminal's font would. Nothing is
//! rasterized here; that is the offload the whole library makes.
//!
//! The Evcxr stdout protocol lives in [`crate::evcxr`], behind its feature.

use super::color::Color;
use super::raster::Raster;
use crate::Theme;

/// Escapes one text-cell glyph into HTML (or SVG) element content.
pub(super) fn escape(glyph: char, out: &mut String) {
    match glyph {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        glyph => out.push(glyph),
    }
}

impl Raster {
    /// Encodes the cell grid as HTML element content with concrete-RGB span runs.
    ///
    /// Default-colored glyphs inherit from their enclosing element. Rows are
    /// newline-joined with trailing spaces trimmed, just like [`Raster::encode`].
    pub(crate) fn encode_html(&self) -> String {
        use std::fmt::Write as _;

        let mut out = String::with_capacity((self.width() + 32) * self.height());
        for row in 0..self.height() {
            if row > 0 {
                out.push('\n');
            }
            let mut current = (None, None);
            let mut kept = out.len();
            let mut kept_style = (None, None);
            for cell in self.row(row) {
                let foreground = match cell.foreground {
                    Color::Default => None,
                    color => Some(color.to_rgb()),
                };
                let background = match cell.background {
                    Color::Default => None,
                    color => Some(color.to_rgb()),
                };
                // Foreground is immaterial on a space, but its background is not.
                let next = (
                    if cell.glyph == ' ' {
                        current.0
                    } else {
                        foreground
                    },
                    background,
                );
                if next != current {
                    if current != (None, None) {
                        out.push_str("</span>");
                    }
                    if next != (None, None) {
                        out.push_str("<span style=\"");
                        if let Some((r, g, b)) = next.0 {
                            let _ = write!(out, "color:#{r:02x}{g:02x}{b:02x}");
                            if next.1.is_some() {
                                out.push(';');
                            }
                        }
                        if let Some((r, g, b)) = next.1 {
                            let _ = write!(out, "background-color:#{r:02x}{g:02x}{b:02x}");
                        }
                        out.push_str("\">");
                    }
                    current = next;
                }
                escape(cell.glyph, &mut out);
                if cell.glyph != ' ' || background.is_some() {
                    kept = out.len();
                    kept_style = current;
                }
            }
            out.truncate(kept);
            if kept_style != (None, None) {
                out.push_str("</span>");
            }
        }
        out
    }

    /// Encodes the raster as a self-contained HTML terminal card.
    ///
    /// The cell grid sits in a styled `<pre>`: default-colored chrome inherits
    /// the card foreground, mark colors become concrete-RGB spans, and the card
    /// background and foreground follow `theme` ([`Theme::LIGHT`] selects the
    /// light card; every other theme the dark one). A plot's
    /// [`crate::Plot::to_html`] is this encoding of its raster. Pure and
    /// deterministic: the same raster and theme always produce the same string.
    pub fn to_html(&self, theme: Theme) -> String {
        use std::fmt::Write as _;

        let content = self.encode_html();
        let (background, foreground) = theme.card_colors();
        let mut html = String::with_capacity(content.len() + 320);
        let _ = write!(
            html,
            "<pre style=\"margin:0;padding:12px 16px;border:0;border-radius:8px;box-sizing:border-box;display:inline-block;max-width:100%;overflow-x:auto;white-space:pre;font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:13px;line-height:1.1;font-variant-ligatures:none;font-feature-settings:\"liga\" 0,\"calt\" 0;background-color:{background};color:{foreground}\">{content}</pre>"
        );
        html
    }
}

#[cfg(test)]
#[path = "tests/html_tests.rs"]
mod tests;
