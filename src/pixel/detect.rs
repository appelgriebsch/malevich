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

/// The protocols the environment advertises, best first. Pure over its
/// lookup, so tests need no process-global environment mutation.
pub(crate) fn sniff(variable: &impl Fn(&str) -> Option<String>) -> Vec<Protocol> {
    // Multiplexers sit between us and the terminal: without passthrough
    // handling, an image escape would be swallowed or mangled.
    if variable("TMUX").is_some() {
        return Vec::new();
    }
    let term = variable("TERM").unwrap_or_default();
    if term == "dumb" || term.starts_with("screen") || term.starts_with("tmux") {
        return Vec::new();
    }
    if variable("KITTY_WINDOW_ID").is_some() || term.contains("kitty") || term.contains("ghostty") {
        return vec![Protocol::Kitty];
    }
    let program = variable("TERM_PROGRAM")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if program.contains("ghostty") {
        return vec![Protocol::Kitty];
    }
    // Both also speak sixel; their native inline-image protocol ranks first
    // because it pins the panel to its cell box, which sixel cannot.
    if program.contains("iterm") || program.contains("wezterm") {
        return vec![Protocol::ITerm2, Protocol::Sixel];
    }
    if term.contains("foot") {
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
