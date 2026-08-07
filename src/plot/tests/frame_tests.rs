use super::{detect_charset_with, named_charset};
use crate::Charset;

fn detect(variables: &[(&str, &str)]) -> Charset {
    detect_charset_with(|name| {
        variables
            .iter()
            .find_map(|(key, value)| (*key == name).then(|| (*value).to_string()))
    })
}

#[test]
fn utf8_auto_detection_is_conservatively_quadrants() {
    assert_eq!(detect(&[]), Charset::Quadrants);
    assert_eq!(detect(&[("LANG", "en_US.UTF-8")]), Charset::Quadrants);

    // Terminal identity is deliberately not treated as proof of font coverage.
    for variables in [
        &[("TERM", "xterm-kitty")][..],
        &[("KITTY_WINDOW_ID", "1")][..],
        &[("TERM_PROGRAM", "ghostty")][..],
        &[("TERM", "xterm-256color"), ("WT_SESSION", "id")][..],
        &[("VTE_VERSION", "9999")][..],
    ] {
        assert_eq!(detect(variables), Charset::Quadrants, "{variables:?}");
    }
}

#[test]
fn hostile_or_non_utf8_environments_fall_back_to_ascii() {
    assert_eq!(detect(&[("TERM", "dumb")]), Charset::Ascii);
    assert_eq!(detect(&[("LC_ALL", "C")]), Charset::Ascii);
    assert_eq!(
        detect(&[("LC_ALL", "C"), ("LANG", "en_US.UTF-8")]),
        Charset::Ascii
    );
    assert_eq!(detect(&[("LC_CTYPE", "POSIX")]), Charset::Ascii);
}

#[test]
fn explicit_charset_override_has_highest_precedence() {
    for (name, charset) in [
        ("ascii", Charset::Ascii),
        ("half", Charset::HalfBlocks),
        ("quad", Charset::Quadrants),
        ("sextant", Charset::Sextants),
        ("octant", Charset::Octants),
        ("braille", Charset::Braille),
    ] {
        assert_eq!(
            detect(&[("MALEVICH_CHARSET", name), ("TERM", "dumb")]),
            charset,
            "{name}"
        );
    }
    assert_eq!(detect(&[("MALEVICH_CHARSET", "bogus")]), Charset::Quadrants);
    assert_eq!(detect(&[("MALEVICH_CHARSET", "auto")]), Charset::Quadrants);
}

#[test]
fn charset_override_accepts_readable_aliases() {
    assert_eq!(named_charset(" Quadrants "), Some(Charset::Quadrants));
    assert_eq!(named_charset("HALFBLOCKS"), Some(Charset::HalfBlocks));
    assert_eq!(named_charset("octants"), Some(Charset::Octants));
    assert_eq!(named_charset("unknown"), None);
}

#[test]
fn portable_frame_is_deterministic_and_uses_old_block_elements() {
    let frame = super::Frame::portable(20, 8);
    assert_eq!(frame.charset, Charset::Quadrants);
    assert_eq!(frame.color, crate::ColorMode::Plain);
    assert_eq!(frame.theme, crate::Theme::DARK);
}
