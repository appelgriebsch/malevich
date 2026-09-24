use std::collections::HashMap;

use super::sniff;
use crate::pixel::Protocol;

fn environment(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let map: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    move |name: &str| map.get(name).cloned()
}

type Case<'a> = (&'a [(&'a str, &'a str)], &'a [Protocol]);

#[test]
fn known_terminals_map_to_their_protocols_best_first() {
    let cases: &[Case] = &[
        (&[("KITTY_WINDOW_ID", "1")], &[Protocol::Kitty]),
        (&[("TERM", "xterm-kitty")], &[Protocol::Kitty]),
        (&[("TERM_PROGRAM", "ghostty")], &[Protocol::Kitty]),
        (
            &[("TERM_PROGRAM", "iTerm.app")],
            &[Protocol::ITerm2, Protocol::Sixel],
        ),
        (
            &[("TERM_PROGRAM", "WezTerm")],
            &[Protocol::ITerm2, Protocol::Sixel],
        ),
        (&[("TERM", "foot-extra")], &[Protocol::Sixel]),
        (&[("KONSOLE_VERSION", "230400")], &[Protocol::Sixel]),
        (&[("WT_SESSION", "guid")], &[Protocol::Sixel]),
        (&[("KITTY_PID", "4242")], &[Protocol::Kitty]),
        (
            &[("GHOSTTY_BIN_DIR", "/opt/ghostty/bin")],
            &[Protocol::Kitty],
        ),
        (
            &[("LC_TERMINAL", "iTerm2"), ("TERM", "xterm-256color")],
            &[Protocol::ITerm2, Protocol::Sixel],
        ),
        (
            &[("WEZTERM_EXECUTABLE", "/usr/bin/wezterm")],
            &[Protocol::ITerm2, Protocol::Sixel],
        ),
        (
            &[("TERM_PROGRAM", "rio")],
            &[Protocol::ITerm2, Protocol::Sixel],
        ),
        (&[("TERM_PROGRAM", "WarpTerminal")], &[Protocol::ITerm2]),
        (&[("MLTERM", "3.9.3")], &[Protocol::Sixel]),
    ];
    for (pairs, expected) in cases {
        assert_eq!(sniff(&environment(pairs)), *expected, "{pairs:?}");
    }
}

#[test]
fn unknown_and_hostile_environments_detect_nothing() {
    let cases: &[&[(&str, &str)]] = &[
        &[],
        &[("TERM", "xterm-256color")],
        &[("TERM", "dumb")],
        &[("TERM", "unknown")],
        &[("TERM_PROGRAM", "Apple_Terminal")],
        &[("TERM_PROGRAM", "vscode")],
        &[("KONSOLE_VERSION", "210800")],
    ];
    for pairs in cases {
        assert_eq!(sniff(&environment(pairs)), Vec::new(), "{pairs:?}");
    }
}

#[test]
fn multiplexers_suppress_detection_even_inside_a_capable_terminal() {
    assert_eq!(
        sniff(&environment(&[
            ("TMUX", "/tmp/tmux-1000/default,1234,0"),
            ("KITTY_WINDOW_ID", "1"),
        ])),
        Vec::new()
    );
    assert_eq!(
        sniff(&environment(&[
            ("TERM", "screen-256color"),
            ("TERM_PROGRAM", "iTerm.app"),
        ])),
        Vec::new()
    );
}

#[test]
fn an_explicit_graphics_override_outranks_the_sniff() {
    // Named protocols, even under a multiplexer the user has configured.
    assert_eq!(
        sniff(&environment(&[
            ("MALEVICH_GRAPHICS", "kitty"),
            ("TMUX", "/tmp/tmux-1000/default,1234,0"),
        ])),
        vec![Protocol::Kitty]
    );
    assert_eq!(
        sniff(&environment(&[("MALEVICH_GRAPHICS", "iterm2")])),
        vec![Protocol::ITerm2]
    );
    assert_eq!(
        sniff(&environment(&[("MALEVICH_GRAPHICS", "Sixel")])),
        vec![Protocol::Sixel]
    );
    // `none` is cells inside the most capable terminal.
    assert_eq!(
        sniff(&environment(&[
            ("MALEVICH_GRAPHICS", "none"),
            ("KITTY_WINDOW_ID", "1"),
        ])),
        Vec::new()
    );
    // A name nobody knows is ignored, not obeyed.
    assert_eq!(
        sniff(&environment(&[
            ("MALEVICH_GRAPHICS", "hologram"),
            ("KITTY_WINDOW_ID", "1"),
        ])),
        vec![Protocol::Kitty]
    );
}
