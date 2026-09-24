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
///
/// The fields are public, so a detected frame at an explicit size — the
/// terminal's charset, color tier, and theme, your dimensions — is struct update
/// syntax, not a builder:
///
/// ```
/// use malevich::Frame;
///
/// let frame = Frame {
///     width: 100,
///     height: 30,
///     ..Frame::detect()
/// };
/// assert_eq!((frame.width, frame.height), (100, 30));
/// ```
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
    /// The legacy deterministic frame: braille glyphs, no color.
    ///
    /// This remains unchanged for 1.x snapshot compatibility. For user-facing text
    /// and files, prefer [`Frame::portable`], whose older block-element glyphs have
    /// broader font coverage.
    pub fn plain(width: usize, height: usize) -> Frame {
        Frame {
            width,
            height,
            charset: Charset::Braille,
            color: ColorMode::Plain,
            theme: Theme::DARK,
        }
    }

    /// A deterministic, conservative Unicode frame: quadrants, no color.
    ///
    /// Quadrants use the long-established Block Elements range and are the default
    /// automatic tier for UTF-8 environments. Use an explicit ASCII frame when the
    /// destination's Unicode support is unknown.
    pub fn portable(width: usize, height: usize) -> Frame {
        Frame {
            width,
            height,
            charset: Charset::Quadrants,
            color: ColorMode::Plain,
            theme: Theme::DARK,
        }
    }

    /// Detects a frame for stdout — [`Frame::detect_for`] against
    /// [`std::io::stdout`].
    ///
    /// Size: the terminal's width and about a third of its height; without a
    /// terminal, `COLUMNS` and `LINES` when set, else 80×16. Charset: an
    /// explicit `MALEVICH_CHARSET` override, otherwise ASCII for `TERM=dumb`
    /// or `unknown` or an explicitly non-UTF-8 locale, and quadrants for
    /// UTF-8. Dense Unicode tiers are opt-in because terminal identity cannot
    /// establish font coverage. Color, in precedence order: `NO_COLOR`
    /// (non-empty) disables; `CLICOLOR_FORCE` or `FORCE_COLOR` (non-empty, not
    /// `0`) forces color even when piped; otherwise color only when stdout is
    /// a terminal — at the tier named by `COLORTERM=truecolor` or a
    /// `TERM=*-direct`, a `256color` `TERM`, or 16-color ANSI as the floor,
    /// with `TERM=screen*` capped at 256.
    pub fn detect() -> Frame {
        Frame::detect_for(&std::io::stdout())
    }

    /// Detects a frame for a specific destination — the same ladder as
    /// [`Frame::detect`], but with the color gate keyed to `destination`'s
    /// tty-ness instead of stdout's.
    ///
    /// A tool that writes its plot to stderr (leaving stdout for data) must detect
    /// against stderr: `Frame::detect_for(&std::io::stderr())`. Detecting against
    /// stdout there would read the wrong stream — a piped stdout would strip color
    /// from a plot going to a live terminal. `NO_COLOR` / `CLICOLOR_FORCE` /
    /// `TERM=dumb` keep their precedence regardless of destination. Size still comes
    /// from whichever of stdout/stderr/stdin is a terminal.
    pub fn detect_for(destination: &impl IsTerminal) -> Frame {
        let measured = terminal_size::terminal_size()
            .map(|(terminal_size::Width(w), terminal_size::Height(h))| (w as usize, h as usize));
        let (width, height) = detect_size_with(measured, variable);
        Frame {
            width,
            height,
            charset: detect_charset(),
            color: detect_color_with(destination.is_terminal(), variable),
            theme: Theme::detect(),
        }
    }
}

fn variable(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

/// The frame size: the measured terminal, a third of its height; without one,
/// `COLUMNS` and `LINES` when they parse (a piped render inside a shell that
/// exports them), else 80×16.
fn detect_size_with(
    measured: Option<(usize, usize)>,
    mut variable: impl FnMut(&str) -> Option<String>,
) -> (usize, usize) {
    let rows_to_height = |rows: usize| (rows / 3).clamp(8, 24);
    if let Some((width, rows)) = measured {
        return (width, rows_to_height(rows));
    }
    let mut exported = |name: &str| {
        variable(name)
            .and_then(|value| value.trim().parse::<usize>().ok())
            .filter(|value| *value > 0)
    };
    let width = exported("COLUMNS").unwrap_or(80);
    let height = exported("LINES").map_or(16, rows_to_height);
    (width, height)
}

/// A `TERM` that declares no capabilities at all.
fn is_dumb(term: &str) -> bool {
    term == "dumb" || term == "unknown"
}

fn detect_charset() -> Charset {
    detect_charset_with(variable)
}

fn detect_charset_with(mut variable: impl FnMut(&str) -> Option<String>) -> Charset {
    if let Some(charset) = variable("MALEVICH_CHARSET").and_then(|value| named_charset(&value)) {
        return charset;
    }
    if variable("TERM").is_some_and(|term| is_dumb(&term)) {
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
    Charset::Quadrants
}

fn named_charset(value: &str) -> Option<Charset> {
    Some(match value.trim().to_ascii_lowercase().as_str() {
        "ascii" => Charset::Ascii,
        "half" | "halfblock" | "halfblocks" => Charset::HalfBlocks,
        "quad" | "quadrant" | "quadrants" => Charset::Quadrants,
        "sextant" | "sextants" => Charset::Sextants,
        "octant" | "octants" => Charset::Octants,
        "braille" => Charset::Braille,
        "auto" => return None,
        _ => return None,
    })
}

/// The color tier, pure over its lookup. `NO_COLOR` wins; `CLICOLOR_FORCE` or
/// `FORCE_COLOR` (non-empty, not `0`) keeps color off a terminal; a dumb or
/// unknown `TERM` is plain; `COLORTERM=truecolor` or a `*-direct` `TERM` is
/// truecolor; a `screen*` `TERM` caps at 256, since the multiplexer
/// re-encodes what passes through it; `256color` is 256; the floor is 16.
fn detect_color_with(
    is_terminal: bool,
    mut variable: impl FnMut(&str) -> Option<String>,
) -> ColorMode {
    if variable("NO_COLOR").is_some() {
        return ColorMode::Plain;
    }
    let forcing = |value: String| value != "0";
    let forced = variable("CLICOLOR_FORCE").is_some_and(forcing)
        || variable("FORCE_COLOR").is_some_and(forcing);
    if !forced && !is_terminal {
        return ColorMode::Plain;
    }
    let term = variable("TERM").unwrap_or_default();
    if is_dumb(&term) {
        return ColorMode::Plain;
    }
    let colorterm = variable("COLORTERM").unwrap_or_default();
    let truecolor = colorterm == "truecolor" || colorterm == "24bit" || term.ends_with("-direct");
    if term.starts_with("screen") {
        return if truecolor || term.contains("256color") {
            ColorMode::Ansi256
        } else {
            ColorMode::Ansi16
        };
    }
    if truecolor {
        return ColorMode::TrueColor;
    }
    if term.contains("256color") {
        return ColorMode::Ansi256;
    }
    ColorMode::Ansi16
}

#[cfg(test)]
#[path = "tests/frame_tests.rs"]
mod tests;
