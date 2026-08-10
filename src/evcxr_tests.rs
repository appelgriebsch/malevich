use super::{card_colors, mime_bundle};
use crate::{Color, Theme};

#[test]
fn card_colors_follow_light_and_dark_themes() {
    assert_eq!(card_colors(Theme::LIGHT), ("#ffffff", "#1f2328"));
    assert_eq!(card_colors(Theme::DARK), ("#0d1117", "#e6edf3"));

    let custom = Theme {
        palette: [Color::BrightRed; 6],
    };
    assert_eq!(card_colors(custom), card_colors(Theme::DARK));
}

#[test]
fn the_mime_bundle_wraps_alternative_representations() {
    // Two blocks joined by a single newline; the frontend picks the richest it can
    // render (Jupyter -> html, terminal REPL -> plain).
    assert_eq!(
        mime_bundle(&[("text/html", "<pre>plot</pre>"), ("text/plain", "plot")]),
        "EVCXR_BEGIN_CONTENT text/html\n<pre>plot</pre>\nEVCXR_END_CONTENT\n\
         EVCXR_BEGIN_CONTENT text/plain\nplot\nEVCXR_END_CONTENT"
    );
}

#[test]
fn one_block_carries_no_separator() {
    assert_eq!(
        mime_bundle(&[("text/plain", "plot")]),
        "EVCXR_BEGIN_CONTENT text/plain\nplot\nEVCXR_END_CONTENT"
    );
}

#[test]
fn an_empty_bundle_renders_nothing() {
    assert_eq!(mime_bundle(&[]), "");
}

#[test]
fn content_is_passed_through_so_callers_own_their_escaping() {
    // The protocol is a framing, not a sanitizer: a caller emitting text
    // rather than markup escapes it before handing it over.
    assert_eq!(
        mime_bundle(&[("text/html", "a < b")]),
        "EVCXR_BEGIN_CONTENT text/html\na < b\nEVCXR_END_CONTENT"
    );
}

#[test]
fn the_card_a_plot_draws_matches_the_exported_colors() {
    // The whole point of exporting these: a consumer's own card and a
    // malevich plot card must not disagree inside one notebook cell.
    let plot = crate::Plot::new().layer(crate::Line::y(vec![1.0, 2.0]));
    for theme in [Theme::DARK, Theme::LIGHT] {
        let mut frame = crate::Frame::plain(20, 6);
        frame.theme = theme;
        let (background, foreground) = card_colors(theme);
        let html = plot.to_html(&frame);
        assert!(html.contains(&format!("background-color:{background}")));
        assert!(html.contains(&format!("color:{foreground}")));
    }
}
