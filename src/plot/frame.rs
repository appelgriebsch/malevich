//! `Frame`: where and how to render — the only place the crate looks at the
//! environment.

use std::io::IsTerminal;

use crate::render::Charset;

/// How much color the output may carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// No escape codes at all: safe for files, pipes, and logs.
    Plain,
    /// The 16-color ANSI palette.
    Ansi,
}

/// Where and how to render: size in cells, charset, and color mode.
///
/// A frame is render state, not plot state — the same [`crate::Plot`] renders into
/// many frames. Rendering is deterministic: the same plot and the same frame always
/// produce the same string. [`Frame::detect`] is the single place environment
/// inspection happens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    /// Total width in cells, chrome included.
    pub width: usize,
    /// Total height in cells, chrome included.
    pub height: usize,
    /// The glyph tier to encode with.
    pub charset: Charset,
    /// How much color the output may carry.
    pub color: ColorMode,
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
        }
    }

    /// Detects a frame from the environment: terminal width, a height around a third
    /// of the terminal, color only when stdout is a terminal and `NO_COLOR` is unset
    /// (or empty). Falls back to 80×16 plain when there is no terminal.
    pub fn detect() -> Frame {
        let (width, height) = match terminal_size::terminal_size() {
            Some((terminal_size::Width(w), terminal_size::Height(h))) => {
                (w as usize, (h as usize / 3).clamp(8, 24))
            }
            None => (80, 16),
        };
        let no_color = std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty());
        let color = if !no_color && std::io::stdout().is_terminal() {
            ColorMode::Ansi
        } else {
            ColorMode::Plain
        };
        Frame {
            width,
            height,
            charset: Charset::Braille,
            color,
        }
    }
}
