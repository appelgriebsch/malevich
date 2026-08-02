use super::{display_width, fit_width_with};

#[test]
fn wide_glyphs_measure_two_columns() {
    assert_eq!(display_width("abc"), 3);
    assert_eq!(display_width("\u{65E5}\u{672C}"), 4);
    assert_eq!(display_width("a\u{65E5}b"), 4);
}

#[test]
fn combining_marks_measure_nothing() {
    assert_eq!(display_width("e\u{0301}"), 1);
}

#[test]
fn fitting_keeps_short_text_untouched() {
    assert_eq!(fit_width_with("hi", 4, '\u{2026}'), "hi");
    assert_eq!(fit_width_with("", 0, '\u{2026}'), "");
}

#[test]
fn fitting_truncates_with_an_ellipsis() {
    assert_eq!(fit_width_with("hello", 4, '\u{2026}'), "hel\u{2026}");
    assert_eq!(fit_width_with("hello", 1, '\u{2026}'), "\u{2026}");
}

#[test]
fn fitting_never_splits_a_wide_glyph() {
    // Four columns: the second ideograph (2 columns) cannot fit next to the
    // ellipsis, so it is dropped whole.
    assert_eq!(
        fit_width_with("\u{65E5}\u{672C}\u{8A9E}", 4, '\u{2026}'),
        "\u{65E5}\u{2026}"
    );
}
