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

#[test]
fn known_terminals_map_to_their_best_protocol() {
    let cases: &[(&[(&str, &str)], Protocol)] = &[
        (&[("KITTY_WINDOW_ID", "1")], Protocol::Kitty),
        (&[("TERM", "xterm-kitty")], Protocol::Kitty),
        (&[("TERM_PROGRAM", "ghostty")], Protocol::Kitty),
        (&[("TERM_PROGRAM", "iTerm.app")], Protocol::ITerm2),
        (&[("TERM_PROGRAM", "WezTerm")], Protocol::ITerm2),
        (&[("TERM", "foot-extra")], Protocol::Sixel),
        (&[("KONSOLE_VERSION", "230400")], Protocol::Sixel),
        (&[("WT_SESSION", "guid")], Protocol::Sixel),
    ];
    for (pairs, expected) in cases {
        assert_eq!(sniff(environment(pairs)), Some(*expected), "{pairs:?}");
    }
}

#[test]
fn unknown_and_hostile_environments_detect_nothing() {
    let cases: &[&[(&str, &str)]] = &[
        &[],
        &[("TERM", "xterm-256color")],
        &[("TERM", "dumb")],
        &[("TERM_PROGRAM", "Apple_Terminal")],
        &[("TERM_PROGRAM", "vscode")],
        &[("KONSOLE_VERSION", "210800")],
    ];
    for pairs in cases {
        assert_eq!(sniff(environment(pairs)), None, "{pairs:?}");
    }
}

#[test]
fn multiplexers_suppress_detection_even_inside_a_capable_terminal() {
    assert_eq!(
        sniff(environment(&[
            ("TMUX", "/tmp/tmux-1000/default,1234,0"),
            ("KITTY_WINDOW_ID", "1"),
        ])),
        None
    );
    assert_eq!(
        sniff(environment(&[
            ("TERM", "screen-256color"),
            ("TERM_PROGRAM", "iTerm.app"),
        ])),
        None
    );
}
