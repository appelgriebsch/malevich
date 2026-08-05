//! Live mode (`--live`): read stdin forever, repaint a sliding window in place.
//!
//! This is the library's [`stream`](malevich::stream) module, exposed: a
//! thread-shared [`Ring`] the reader fills while the render loop takes cheap
//! snapshots, [`Rate`] to turn a monotonic counter into per-sample deltas, and
//! [`Live`] for the flicker-free cursor-up/erase-down repaint (no alt-screen, so
//! the final frame survives in scrollback). The cursor is hidden while repainting
//! and restored on EOF, SIGINT, or a closed pipe.
//!
//! The frame is re-detected every repaint, so a terminal resize degrades to a
//! clean redraw at the new size.

use std::io::{self, BufRead, IsTerminal, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use malevich::stream::{Live, Rate, Ring};
use malevich::{Line, Plot};

use crate::args::{Args, Output};
use crate::output;

/// Set by the SIGINT handler; the render loop stops at the next frame and the
/// cursor is restored — the whole reason a bare Ctrl-C is intercepted.
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Runs the live loop for `args` (validated to be `line`, terminal destination).
pub fn run(args: &Args) -> io::Result<()> {
    install_interrupt_handler();
    // `io::stdout`/`io::stderr` are cheap factories for a fresh handle to the same
    // shared stream — used both for the repaint writer and for cursor control.
    match args.output {
        Output::Stdout => drive(io::stdout, args),
        // Stderr is the default and the file case is rejected during parsing.
        _ => drive(io::stderr, args),
    }
}

/// Drives the reader thread and the repaint loop over a destination factory.
fn drive<W: Write + IsTerminal>(handle: fn() -> W, args: &Args) -> io::Result<()> {
    // Size the window from the frame width once; the loop re-detects size every
    // repaint so resizes are followed.
    let window = args
        .window
        .unwrap_or_else(|| output::frame_for(&handle(), args).width.max(1))
        .max(1);
    let fps = args.fps.unwrap_or(10).max(1);
    // Never zero: past 1000 fps the throttle bottoms out at 1 ms, not a busy spin.
    let interval = Duration::from_millis((1000 / fps as u64).max(1));

    let ring = Ring::new(window);
    let done = spawn_reader(ring.clone(), args.delimiter, args.rate);

    // Hide the cursor for the duration of the repaint (restored below no matter how
    // the loop ends — EOF, interrupt, or a broken pipe).
    let mut cursor = handle();
    let _ = write!(cursor, "\x1b[?25l");
    let _ = cursor.flush();

    let result = repaint(handle, &ring, args, done, interval);

    let _ = write!(cursor, "\x1b[?25h");
    let _ = cursor.flush();

    match result {
        // A closed terminal (SIGPIPE → EPIPE) is a clean stop.
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        other => other,
    }
}

/// The repaint loop: snapshot, draw, throttle — until EOF, interrupt, or error.
fn repaint<W: Write + IsTerminal>(
    handle: fn() -> W,
    ring: &Ring,
    args: &Args,
    done: Arc<AtomicBool>,
    interval: Duration,
) -> io::Result<()> {
    let mut live = Live::new(handle());
    loop {
        let frame = output::frame_for(&handle(), args);
        let plot = plot(ring.snapshot(), args);
        live.draw(&plot, &frame)?;
        // Draw the latest frame, then stop once the input is exhausted or Ctrl-C
        // arrived — the last frame reflects the complete window.
        if done.load(Ordering::Relaxed) || INTERRUPTED.load(Ordering::Relaxed) {
            return Ok(());
        }
        thread::sleep(interval);
    }
}

/// Builds the live `line` plot from the window, applying the furniture that makes
/// sense for a sliding index axis (title, labels, y limits and log). The x axis is
/// the moving window, so x-domain, x-log, and time-x are deliberately not applied.
fn plot(values: Vec<f64>, args: &Args) -> Plot<'static> {
    let mut plot = Plot::new().layer(Line::y(values));
    if let Some(title) = &args.title {
        plot = plot.title(title);
    }
    if let Some(xlabel) = &args.xlabel {
        plot = plot.x_label(xlabel);
    }
    if let Some(ylabel) = &args.ylabel {
        plot = plot.y_label(ylabel);
    }
    if let Some((lo, hi)) = args.ylim {
        plot = plot.y_domain(lo, hi);
    }
    if args.log_y {
        plot = plot.log_y();
    }
    plot
}

/// Spawns the reader: one value per input line into `ring`, forever. Returns a flag
/// it raises at EOF so the render loop can draw a final frame and stop.
fn spawn_reader(ring: Ring, delimiter: Option<char>, rate: bool) -> Arc<AtomicBool> {
    let done = Arc::new(AtomicBool::new(false));
    let eof = done.clone();
    thread::spawn(move || {
        let stdin = io::stdin();
        let mut rate_tracker = Rate::new();
        for line in stdin.lock().lines() {
            let Ok(line) = line else { break };
            if line.trim().is_empty() {
                continue;
            }
            // A non-numeric line is a missed sample — an honest gap, not a value.
            let sample = first_number(&line, delimiter).unwrap_or(f64::NAN);
            let value = if rate {
                rate_tracker.delta(sample)
            } else {
                sample
            };
            ring.push(value);
        }
        eof.store(true, Ordering::Relaxed);
    });
    done
}

/// The first field of `line` that parses as a finite number.
fn first_number(line: &str, delimiter: Option<char>) -> Option<f64> {
    let fields: Box<dyn Iterator<Item = &str>> = match delimiter {
        Some(sep) => Box::new(line.split(sep)),
        None => Box::new(line.split_whitespace()),
    };
    fields
        .filter_map(|field| field.trim().parse::<f64>().ok())
        .find(|value| value.is_finite())
}

/// Installs a SIGINT handler that flips [`INTERRUPTED`]; the render loop notices it
/// within a frame and restores the cursor on the way out. On non-unix targets the
/// default handling applies (documented; the live path targets unix terminals).
fn install_interrupt_handler() {
    #[cfg(unix)]
    {
        extern "C" fn on_interrupt(_signal: libc::c_int) {
            INTERRUPTED.store(true, Ordering::Relaxed);
        }
        // SAFETY: registering a signal handler that only stores into an atomic —
        // async-signal-safe — once, at startup, before the reader thread spawns.
        unsafe {
            libc::signal(libc::SIGINT, on_interrupt as usize as libc::sighandler_t);
        }
    }
}

#[cfg(test)]
#[path = "tests/live_tests.rs"]
mod tests;
