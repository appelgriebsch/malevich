use super::{detect_charset_with, detect_color_with, detect_size_with, named_charset};
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

fn color(is_terminal: bool, variables: &[(&str, &str)]) -> crate::ColorMode {
    detect_color_with(is_terminal, |name| {
        variables
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.to_string())
    })
}

#[test]
fn color_detection_reads_what_the_field_agreed_on() {
    use crate::ColorMode;

    // Off a terminal: plain, unless forced — by either convention, and
    // never by a zero.
    assert_eq!(
        color(false, &[("TERM", "xterm-256color")]),
        ColorMode::Plain
    );
    assert_eq!(
        color(false, &[("FORCE_COLOR", "1"), ("TERM", "xterm-256color")]),
        ColorMode::Ansi256
    );
    assert_eq!(
        color(false, &[("FORCE_COLOR", "0"), ("TERM", "xterm-256color")]),
        ColorMode::Plain
    );
    assert_eq!(
        color(false, &[("CLICOLOR_FORCE", "1"), ("TERM", "xterm")]),
        ColorMode::Ansi16
    );
    // NO_COLOR outranks every force.
    assert_eq!(
        color(true, &[("NO_COLOR", "1"), ("FORCE_COLOR", "1")]),
        ColorMode::Plain
    );
    // A direct-color TERM is truecolor; dumb and unknown are plain.
    assert_eq!(
        color(true, &[("TERM", "xterm-direct")]),
        ColorMode::TrueColor
    );
    assert_eq!(
        color(true, &[("TERM", "xterm"), ("COLORTERM", "truecolor")]),
        ColorMode::TrueColor
    );
    assert_eq!(color(true, &[("TERM", "dumb")]), ColorMode::Plain);
    assert_eq!(color(true, &[("TERM", "unknown")]), ColorMode::Plain);
    // screen re-encodes what passes through it: 256 at most.
    assert_eq!(
        color(
            true,
            &[("TERM", "screen-256color"), ("COLORTERM", "truecolor")]
        ),
        ColorMode::Ansi256
    );
    assert_eq!(color(true, &[("TERM", "screen")]), ColorMode::Ansi16);
    assert_eq!(color(true, &[("TERM", "xterm")]), ColorMode::Ansi16);
}

#[test]
fn a_piped_render_is_sized_by_columns_and_lines() {
    let lookup = |variables: &'static [(&str, &str)]| {
        move |name: &str| {
            variables
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| value.to_string())
        }
    };
    assert_eq!(
        detect_size_with(None, lookup(&[("COLUMNS", "120"), ("LINES", "48")])),
        (120, 16)
    );
    assert_eq!(
        detect_size_with(None, lookup(&[("COLUMNS", "132")])),
        (132, 16)
    );
    assert_eq!(
        detect_size_with(None, lookup(&[("COLUMNS", "0"), ("LINES", "x")])),
        (80, 16)
    );
    assert_eq!(detect_size_with(None, lookup(&[])), (80, 16));
    // A measured terminal outranks the exported size.
    assert_eq!(
        detect_size_with(Some((100, 60)), lookup(&[("COLUMNS", "120")])),
        (100, 20)
    );
    assert_eq!(color(true, &[("TERM", "unknown")]), crate::ColorMode::Plain);
}
