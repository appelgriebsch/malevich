//! `Capabilities`: what the terminal can do, as a plain queryable value.
//!
//! Detection has two tiers. Sniffing reads environment variables — free,
//! instant, wrong only by omission. Probing asks the terminal itself (see
//! [`super::probe`]) — ground truth that survives ssh, at the cost of one
//! ~hundred-millisecond round trip, so it runs at most once per process and
//! only where writing escapes is safe: a real tty, no multiplexer between.
//! Everything downstream of the round trip is a pure value, and tests build
//! `Capabilities` without a terminal anywhere near.

use std::io::IsTerminal;
use std::sync::OnceLock;
use std::time::Duration;

use super::{Graphics, Protocol, detect, probe, query};

/// What the terminal can do: the protocols it accepts (best first), its cell
/// size in device pixels when known, and how certain the answer is.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Capabilities {
    /// Protocols the terminal accepts, best first.
    pub protocols: Vec<Protocol>,
    /// The cell size in device pixels `(width, height)`, when known.
    pub cell_size: Option<(u16, u16)>,
    /// How the answer was obtained.
    pub source: Source,
}

/// How a [`Capabilities`] answer was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Source {
    /// The terminal itself answered the probe: ground truth.
    Probed,
    /// Environment variables only — right on known terminals, silent on
    /// unknown ones (probing was unavailable, unsafe, or unanswered).
    Sniffed,
}

impl Capabilities {
    /// Detects stdout's terminal capabilities.
    ///
    /// This is the stdout-oriented convenience form of
    /// [`Capabilities::detect_for`]. An empty `protocols` means cells are the
    /// ceiling; callers wanting one answer use [`Capabilities::best`].
    pub fn detect() -> Capabilities {
        Capabilities::detect_for(&std::io::stdout())
    }

    /// Detects capabilities for the stream that will receive the output.
    ///
    /// The destination's tty status decides whether an active terminal probe is
    /// safe. The probe is cached, so the controlling terminal is asked at most
    /// once per process. Environment sniffing remains the fallback for pipes,
    /// multiplexers, dumb terminals, non-Unix targets, and unanswered probes.
    ///
    /// Use this instead of [`Capabilities::detect`] when rendering somewhere
    /// other than stdout, such as a CLI whose plot goes to stderr. Applications
    /// that already know a remote or secondary terminal's capabilities can build
    /// this plain value directly and skip ambient detection entirely.
    pub fn detect_for(destination: &impl IsTerminal) -> Capabilities {
        let variable = |name: &str| std::env::var(name).ok().filter(|value| !value.is_empty());
        let sniffed = detect::sniff(&variable);
        let fallback = detect::cell_size();
        if !probing_is_safe(destination.is_terminal(), &variable) {
            return resolve(None, sniffed, fallback);
        }
        resolve(probed(), sniffed, fallback)
    }

    /// The auto-enable choice: the best protocol at the known (or default 8×16)
    /// cell size, or `None` when cells are the ceiling.
    pub fn best(&self) -> Option<Graphics> {
        let graphics = Graphics::new(*self.protocols.first()?);
        match self.cell_size {
            Some((width, height)) => Some(graphics.cell_size(width, height)),
            None => Some(graphics),
        }
    }
}

/// Merges a probe report (when one answered) with the sniffed protocols into
/// one ranked answer. Pure: the whole policy is testable without a terminal.
fn resolve(
    report: Option<&probe::Report>,
    sniffed: Vec<Protocol>,
    fallback_cell: Option<(u16, u16)>,
) -> Capabilities {
    let mut protocols = Vec::new();
    let mut source = Source::Sniffed;
    let mut cell_size = None;
    if let Some(report) = report.filter(|report| report.answered) {
        source = Source::Probed;
        cell_size = report.cell_size;
        if report.kitty {
            protocols.push(Protocol::Kitty);
        }
        let name = report
            .terminal
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase();
        // No protocol probe exists for iTerm2 images; the terminal's own name
        // (which, unlike TERM_PROGRAM, survives ssh) is the evidence.
        if ["iterm", "wezterm", "mintty"]
            .iter()
            .any(|t| name.contains(t))
        {
            protocols.push(Protocol::ITerm2);
        }
        if report.sixel {
            protocols.push(Protocol::Sixel);
        }
    }
    for protocol in sniffed {
        if !protocols.contains(&protocol) {
            protocols.push(protocol);
        }
    }
    // One preference order regardless of which tier contributed what.
    protocols.sort_by_key(|protocol| match protocol {
        Protocol::Kitty => 0,
        Protocol::ITerm2 => 1,
        Protocol::Sixel => 2,
    });
    Capabilities {
        protocols,
        cell_size: cell_size.or(fallback_cell),
        source,
    }
}

/// Whether writing probe escapes to the controlling terminal is safe and
/// meaningful: the output destination is a tty, no multiplexer would swallow
/// or mangle the queries, and the terminal is not declared dumb.
fn probing_is_safe(
    destination_is_terminal: bool,
    variable: &impl Fn(&str) -> Option<String>,
) -> bool {
    if !destination_is_terminal || variable("TMUX").is_some() {
        return false;
    }
    let term = variable("TERM").unwrap_or_default();
    term != "dumb" && !term.starts_with("screen") && !term.starts_with("tmux")
}

/// The probe round trip, at most once per process. A cache is the polite
/// option — the probe writes escapes to the user's terminal, and its subject
/// (the terminal) does not change under a running process. This holds
/// environment facts, never plot or render state.
fn probed() -> Option<&'static probe::Report> {
    static PROBE: OnceLock<Option<probe::Report>> = OnceLock::new();
    PROBE
        .get_or_init(|| {
            let replies = query::exchange(probe::QUERIES, Duration::from_millis(300), probe::done)?;
            Some(probe::parse(&replies))
        })
        .as_ref()
}

#[cfg(test)]
#[path = "tests/capabilities_tests.rs"]
mod tests;
