//! The sniffing tier of detection: protocols from environment variables.
//!
//! Free, instant, never touches the terminal, and wrong only by omission —
//! unknown terminals get cells, never garbage. The probing tier
//! ([`super::capabilities`]) asks the terminal itself and merges with this.

use super::{Capabilities, Graphics, Protocol};

impl Graphics {
    /// The best pixel graphics stdout's terminal offers, or `None` when cells
    /// are the ceiling. Sugar for [`Capabilities::detect`]`().best()`.
    ///
    /// Use [`Graphics::detect_for`] for another destination. An explicit
    /// [`Graphics`] value always overrides ambient detection.
    pub fn detect() -> Option<Graphics> {
        Capabilities::detect().best()
    }

    /// The best pixel graphics `destination` offers, or `None` when cells are
    /// the ceiling. Sugar for [`Capabilities::detect_for`]`(destination).best()`.
    pub fn detect_for(destination: &impl std::io::IsTerminal) -> Option<Graphics> {
        Capabilities::detect_for(destination).best()
    }
}

/// The protocol an explicit `MALEVICH_GRAPHICS` names: `kitty`, `sixel`,
/// `iterm2` (or `iterm`), or `none` for cells; anything else is ignored.
pub(crate) fn named_protocols(value: &str) -> Option<Vec<Protocol>> {
    Some(match value.trim().to_ascii_lowercase().as_str() {
        "kitty" => vec![Protocol::Kitty],
        "sixel" => vec![Protocol::Sixel],
        "iterm2" | "iterm" => vec![Protocol::ITerm2],
        "none" | "cells" => Vec::new(),
        _ => return None,
    })
}

/// The protocols the environment advertises, best first. Pure over its
/// lookup, so tests need no process-global environment mutation.
pub(crate) fn sniff(variable: &impl Fn(&str) -> Option<String>) -> Vec<Protocol> {
    // An explicit override outranks every inference, multiplexers included:
    // the user who sets it has arranged passthrough, or wants cells.
    if let Some(named) = variable("MALEVICH_GRAPHICS").and_then(|value| named_protocols(&value)) {
        return named;
    }
    // Multiplexers sit between us and the terminal: without passthrough
    // handling, an image escape would be swallowed or mangled.
    if variable("TMUX").is_some() {
        return Vec::new();
    }
    let term = variable("TERM").unwrap_or_default();
    if term == "dumb" || term == "unknown" || term.starts_with("screen") || term.starts_with("tmux")
    {
        return Vec::new();
    }
    if variable("KITTY_WINDOW_ID").is_some()
        || variable("KITTY_PID").is_some()
        || variable("GHOSTTY_BIN_DIR").is_some()
        || term.contains("kitty")
        || term.contains("ghostty")
    {
        return vec![Protocol::Kitty];
    }
    let program = variable("TERM_PROGRAM")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if program.contains("ghostty") {
        return vec![Protocol::Kitty];
    }
    // Both also speak sixel; their native inline-image protocol ranks first
    // because it pins the panel to its cell box, which sixel cannot. The
    // program name is set by the terminal for local shells; iTerm2 also
    // exports LC_TERMINAL (which ssh forwards) and WezTerm its executable.
    let iterm = program.contains("iterm")
        || variable("LC_TERMINAL")
            .is_some_and(|value| value.to_ascii_lowercase().contains("iterm"));
    if iterm || program.contains("wezterm") || variable("WEZTERM_EXECUTABLE").is_some() {
        return vec![Protocol::ITerm2, Protocol::Sixel];
    }
    // Rio speaks both; Warp only the inline-image protocol.
    if program.contains("rio") {
        return vec![Protocol::ITerm2, Protocol::Sixel];
    }
    if program.contains("warp") {
        return vec![Protocol::ITerm2];
    }
    if term.contains("foot") || variable("MLTERM").is_some() {
        return vec![Protocol::Sixel];
    }
    // Konsole grew sixel in 22.04; its version variable predates that by years.
    if variable("KONSOLE_VERSION").is_some_and(|v| v.parse::<u32>().is_ok_and(|v| v >= 220400)) {
        return vec![Protocol::Sixel];
    }
    if variable("WT_SESSION").is_some() {
        return vec![Protocol::Sixel];
    }
    Vec::new()
}

/// The cell size in device pixels from the kernel's window size, when the
/// terminal fills in the pixel fields (kitty, iTerm2, WezTerm, foot do).
///
/// Tries the controlling terminal directly, then stdout. The winsize ioctl is a
/// plain syscall — no terminal I/O, so it is safe even when stdout is piped (as
/// under evcxr or a mid-pipeline CLI); without the `/dev/tty` path such a pipe
/// would default to an 8×16 cell and a hairline stroke.
#[cfg(unix)]
pub(crate) fn cell_size() -> Option<(u16, u16)> {
    use std::os::fd::AsFd;

    fn from(fd: impl AsFd) -> Option<(u16, u16)> {
        let size = rustix::termios::tcgetwinsize(fd).ok()?;
        if size.ws_col == 0 || size.ws_row == 0 {
            return None;
        }
        let cell = (size.ws_xpixel / size.ws_col, size.ws_ypixel / size.ws_row);
        // Anything narrower than a hairline is a terminal reporting zeros.
        (cell.0 >= 2 && cell.1 >= 4).then_some(cell)
    }

    std::fs::File::open("/dev/tty")
        .ok()
        .and_then(from)
        .or_else(|| from(std::io::stdout()))
}

#[cfg(not(unix))]
pub(crate) fn cell_size() -> Option<(u16, u16)> {
    None
}

#[cfg(test)]
#[path = "tests/detect_tests.rs"]
mod tests;
