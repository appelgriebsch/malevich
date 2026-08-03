//! Graphics detection: which pixel protocol this terminal speaks, and at what
//! cell size — sniffed from the environment, never probed (active probing via
//! DA1/XTSMGRAPHICS is a planned upgrade; sniffing is free, instant, and wrong
//! only by omission: unknown terminals get cells, never garbage).

use std::io::IsTerminal;

use super::{Graphics, Protocol};

impl Graphics {
    /// Detects pixel graphics support from the environment, or `None` when the
    /// terminal gives no evidence of any — including when stdout is not a
    /// terminal, inside tmux/screen (no passthrough handling yet), and in
    /// emulators whose support is off by default (VS Code). Callers fall back
    /// to cell rendering; an explicit [`Graphics`] value always overrides.
    ///
    /// The cell size comes from the terminal's reported pixel geometry
    /// (`TIOCGWINSZ`), falling back to 8×16 when it reports none.
    pub fn detect() -> Option<Graphics> {
        if !std::io::stdout().is_terminal() {
            return None;
        }
        let protocol = sniff(|name| std::env::var(name).ok().filter(|value| !value.is_empty()))?;
        Some(Graphics {
            protocol,
            cell_size: cell_size().unwrap_or((8, 16)),
        })
    }
}

/// The protocol the environment advertises. Pure over its lookup, so tests
/// need no process-global environment mutation.
fn sniff(variable: impl Fn(&str) -> Option<String>) -> Option<Protocol> {
    // Multiplexers sit between us and the terminal: without passthrough
    // handling, an image escape would be swallowed or mangled.
    if variable("TMUX").is_some() {
        return None;
    }
    let term = variable("TERM").unwrap_or_default();
    if term == "dumb" || term.starts_with("screen") || term.starts_with("tmux") {
        return None;
    }
    if variable("KITTY_WINDOW_ID").is_some() || term.contains("kitty") || term.contains("ghostty") {
        return Some(Protocol::Kitty);
    }
    let program = variable("TERM_PROGRAM")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if program.contains("ghostty") {
        return Some(Protocol::Kitty);
    }
    // iTerm2 and WezTerm both speak sixel too, but their native inline-image
    // protocol pins the panel to its cell box, which sixel cannot.
    if program.contains("iterm") || program.contains("wezterm") {
        return Some(Protocol::ITerm2);
    }
    if term.contains("foot") {
        return Some(Protocol::Sixel);
    }
    // Konsole grew sixel in 22.04; its version variable predates that by years.
    if variable("KONSOLE_VERSION").is_some_and(|v| v.parse::<u32>().is_ok_and(|v| v >= 220400)) {
        return Some(Protocol::Sixel);
    }
    if variable("WT_SESSION").is_some() {
        return Some(Protocol::Sixel);
    }
    None
}

/// The cell size in device pixels from the kernel's window size, when the
/// terminal fills in the pixel fields (kitty, iTerm2, WezTerm, foot do).
#[cfg(unix)]
fn cell_size() -> Option<(u16, u16)> {
    let size = rustix::termios::tcgetwinsize(std::io::stdout()).ok()?;
    if size.ws_col == 0 || size.ws_row == 0 {
        return None;
    }
    let cell = (size.ws_xpixel / size.ws_col, size.ws_ypixel / size.ws_row);
    // Anything narrower than a hairline is a terminal reporting zeros.
    (cell.0 >= 2 && cell.1 >= 4).then_some(cell)
}

#[cfg(not(unix))]
fn cell_size() -> Option<(u16, u16)> {
    None
}

#[cfg(test)]
#[path = "tests/detect_tests.rs"]
mod tests;
