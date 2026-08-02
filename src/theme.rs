//! Themes: plot colors as a plain value you pass — never a global.

use crate::render::Color;

/// The colors a plot draws with, independent of any terminal.
///
/// The default works on dark backgrounds. [`Theme::LIGHT`] swaps out the colors that
/// vanish on white paper-like backgrounds. [`Theme::detect`] picks one from the
/// environment; a custom palette is just a struct literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// Colors assigned to layers that set none explicitly, in layer order.
    pub palette: [Color; 6],
}

impl Theme {
    /// The dark-background palette (the default).
    pub const DARK: Theme = Theme {
        palette: [
            Color::Cyan,
            Color::Yellow,
            Color::Green,
            Color::Magenta,
            Color::Blue,
            Color::Red,
        ],
    };

    /// A palette that stays readable on light backgrounds (no yellow on white).
    pub const LIGHT: Theme = Theme {
        palette: [
            Color::Blue,
            Color::Red,
            Color::Green,
            Color::Magenta,
            Color::Cyan,
            Color::Black,
        ],
    };

    /// Picks a theme from the environment: `COLORFGBG` with a light background
    /// (last segment `7` or `15`) selects [`Theme::LIGHT`]; anything else — including
    /// no information at all — selects [`Theme::DARK`], the safer default.
    pub fn detect() -> Theme {
        let light = std::env::var("COLORFGBG")
            .ok()
            .and_then(|value| value.rsplit(';').next().map(str::to_string))
            .is_some_and(|background| background == "7" || background == "15");
        if light { Theme::LIGHT } else { Theme::DARK }
    }
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::DARK
    }
}
