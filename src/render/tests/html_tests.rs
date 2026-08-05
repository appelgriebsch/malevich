use super::{card, escape, mime_bundle};
use crate::{Color, Theme};

#[test]
fn glyphs_are_escaped_as_html_content() {
    let mut html = String::new();
    for glyph in "<&>\"'".chars() {
        escape(glyph, &mut html);
    }
    assert_eq!(html, "&lt;&amp;&gt;\"'");
}

#[test]
fn card_colors_follow_light_and_dark_themes() {
    assert_eq!(card(Theme::LIGHT), ("#ffffff", "#1f2328"));
    assert_eq!(card(Theme::DARK), ("#0d1117", "#e6edf3"));

    let custom = Theme {
        palette: [Color::BrightRed; 6],
    };
    assert_eq!(card(custom), card(Theme::DARK));
}

#[test]
fn the_mime_bundle_matches_evcxrs_stdout_contract() {
    assert_eq!(
        mime_bundle("<pre>plot</pre>"),
        "EVCXR_BEGIN_CONTENT text/html\n<pre>plot</pre>\nEVCXR_END_CONTENT"
    );
}
