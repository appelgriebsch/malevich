//! `Frame`: where and how to render — the only place the crate looks at the
//! environment.

use std::io::IsTerminal;

use crate::render::{Charset, ColorMode};
use crate::theme::Theme;

/// Where and how to render: size in cells, charset, and color mode.
///
/// A frame is render state, not plot state — the same [`crate::Plot`] renders into
/// many frames. Rendering is deterministic: the same plot and the same frame always
/// produce the same string. [`Frame::detect`] is the single place environment
/// inspection happens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Frame {
    /// Total width in cells, chrome included.
    pub width: usize,
    /// Total height in cells, chrome included.
    pub height: usize,
    /// The glyph tier to encode with.
    pub charset: Charset,
    /// How much color the output may carry.
    pub color: ColorMode,
    /// The colors to draw with.
    pub theme: Theme,
}

impl Frame {
    /// A deterministic frame: braille glyphs, no color. The right choice for tests,
    /// files, and anywhere the environment must not matter.
    pub fn plain(width: usize, height: usize) -> Frame {
        Frame {
            width,
            height,
            charset: Charset::Braille,
            color: ColorMode::Plain,
            theme: Theme::DARK,
        }
    }

    /// Detects a frame from the environment.
    ///
    /// Size: the terminal's width and about a third of its height (80×16 without a
    /// terminal). Charset: ASCII for `TERM=dumb` or an explicitly non-UTF-8 locale,
    /// braille otherwise. Color, in precedence order: `NO_COLOR` (non-empty)
    /// disables; `CLICOLOR_FORCE` (non-empty, not `0`) forces color even when piped;
    /// otherwise color only when stdout is a terminal — at the tier named by
    /// `COLORTERM=truecolor`, a `256color` `TERM`, or 16-color ANSI as the floor.
    pub fn detect() -> Frame {
        let (width, height) = match terminal_size::terminal_size() {
            Some((terminal_size::Width(w), terminal_size::Height(h))) => {
                (w as usize, (h as usize / 3).clamp(8, 24))
            }
            None => (80, 16),
        };
        Frame {
            width,
            height,
            charset: detect_charset(),
            color: detect_color(),
            theme: Theme::detect(),
        }
    }
}

fn variable(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

fn detect_charset() -> Charset {
    if variable("TERM").as_deref() == Some("dumb") {
        return Charset::Ascii;
    }
    // POSIX precedence; the first set variable decides. Unset means a modern
    // default, which means UTF-8.
    for name in ["LC_ALL", "LC_CTYPE", "LANG"] {
        if let Some(locale) = variable(name) {
            if !locale.to_ascii_lowercase().contains("utf") {
                return Charset::Ascii;
            }
            break;
        }
    }
    if renders_octants() {
        Charset::Octants
    } else {
        Charset::Braille
    }
}

/// Whether this terminal is known to draw Unicode 16 octants itself (no font
/// gamble). Glyph support cannot be probed, only recognized — the conservative
/// list: kitty, ghostty, WezTerm, foot, recent VTE, Windows Terminal.
fn renders_octants() -> bool {
    if variable("KITTY_WINDOW_ID").is_some() || variable("WT_SESSION").is_some() {
        return true;
    }
    if let Some(program) = variable("TERM_PROGRAM") {
        let program = program.to_ascii_lowercase();
        if program.contains("ghostty") || program.contains("wezterm") {
            return true;
        }
    }
    if let Some(vte) = variable("VTE_VERSION")
        && vte.parse::<u32>().is_ok_and(|version| version >= 7800)
    {
        return true;
    }
    let term = variable("TERM").unwrap_or_default();
    term.contains("kitty") || term.contains("foot") || term.contains("ghostty")
}

fn detect_color() -> ColorMode {
    if variable("NO_COLOR").is_some() {
        return ColorMode::Plain;
    }
    let forced = variable("CLICOLOR_FORCE").is_some_and(|value| value != "0");
    if !forced && !std::io::stdout().is_terminal() {
        return ColorMode::Plain;
    }
    let term = variable("TERM").unwrap_or_default();
    if term == "dumb" {
        return ColorMode::Plain;
    }
    let colorterm = variable("COLORTERM").unwrap_or_default();
    if colorterm == "truecolor" || colorterm == "24bit" {
        return ColorMode::TrueColor;
    }
    if term.contains("256color") {
        return ColorMode::Ansi256;
    }
    ColorMode::Ansi16
}
