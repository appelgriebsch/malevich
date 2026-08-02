//! The 16-color ANSI palette. Deeper color tiers arrive with the theme work.

/// A foreground color: the terminal's default, or one of the 16 ANSI palette entries.
///
/// The palette names map to SGR codes 30–37 and 90–97; what they look like is the
/// terminal's (and its theme's) decision — which is what makes them safe defaults.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Color {
    #[default]
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl Color {
    /// The SGR foreground code for this color.
    pub(crate) fn sgr(self) -> u8 {
        match self {
            Color::Default => 39,
            Color::Black => 30,
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::White => 37,
            Color::BrightBlack => 90,
            Color::BrightRed => 91,
            Color::BrightGreen => 92,
            Color::BrightYellow => 93,
            Color::BrightBlue => 94,
            Color::BrightMagenta => 95,
            Color::BrightCyan => 96,
            Color::BrightWhite => 97,
        }
    }
}
