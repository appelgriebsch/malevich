//! Live mode (`--live`): read stdin forever, repaint a sliding window in place —
//! one line per numeric field of the input, or every value since the start
//! with `--window 0`.
//!
//! This is the library's [`stream`](malevich::stream) module, exposed: a
//! thread-shared [`Ring`] the reader fills while the render loop takes cheap
//! snapshots, [`Rate`] to turn a monotonic counter into per-sample deltas, and
//! [`Live`] for the flicker-free cursor-up/erase-down repaint (no alt-screen, so
//! the final frame survives in scrollback). The cursor is hidden while repainting
//! and restored on EOF, SIGINT, or a closed pipe. A destination that is not a
//! terminal (`2>log`) receives the frames as plain text and no escape byte at
//! all — the library's `Live::detect` decides, and the cursor is left alone.
//!
//! The frame is re-detected every repaint, so a terminal resize degrades to a
//! clean redraw at the new size.

use std::io::{self, BufRead, IsTerminal, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use std::sync::Mutex;

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
    // `--window 0` is the growing window: every value since the start.
    let window = args
        .window
        .unwrap_or_else(|| output::frame_for(&handle(), args).width.max(1));
    let fps = args.fps.unwrap_or(10).max(1);
    // Never zero: past 1000 fps the throttle bottoms out at 1 ms, not a busy spin.
    let interval = Duration::from_millis((1000 / fps as u64).max(1));

    let rings = Rings::new(window);
    let done = spawn_reader(rings.clone(), args.delimiter, args.rate);

    // Hide the cursor for the duration of the repaint (restored below no matter how
    // the loop ends — EOF, interrupt, or a broken pipe) — only where the
    // destination is a terminal: a redirected stream gets no escapes.
    let mut cursor = handle();
    let terminal = cursor.is_terminal();
    if terminal {
        let _ = write!(cursor, "\x1b[?25l");
        let _ = cursor.flush();
    }

    let result = repaint(handle, &rings, args, done, interval);

    if terminal {
        let _ = write!(cursor, "\x1b[?25h");
        let _ = cursor.flush();
    }

    match result {
        // A closed terminal (SIGPIPE → EPIPE) is a clean stop.
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        other => other,
    }
}

/// The repaint loop: snapshot, draw, throttle — until EOF, interrupt, or error.
fn repaint<W: Write + IsTerminal>(
    handle: fn() -> W,
    rings: &Rings,
    args: &Args,
    done: Arc<AtomicBool>,
    interval: Duration,
) -> io::Result<()> {
    let mut live = Live::detect(handle());
    loop {
        // Read the flag before the snapshot: the reader raises it after its
        // last push, so a frame drawn after seeing it holds the complete
        // window — the last frame, once the input is exhausted or Ctrl-C
        // arrived.
        let finished = done.load(Ordering::Relaxed) || INTERRUPTED.load(Ordering::Relaxed);
        let frame = output::frame_for(&handle(), args);
        let plot = plot(rings.snapshot(), args);
        live.draw(&plot, &frame)?;
        if finished {
            return Ok(());
        }
        thread::sleep(interval);
    }
}

/// One window per numeric field of the input, sized alike, shared with the
/// reader. The field count is the first sample's; later samples pad with gaps
/// or drop extras, so every series stays aligned by sample index.
#[derive(Clone)]
struct Rings {
    window: usize,
    rings: Arc<Mutex<Vec<Ring>>>,
}

impl Rings {
    fn new(window: usize) -> Rings {
        Rings {
            window,
            rings: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn ring(&self) -> Ring {
        if self.window == 0 {
            Ring::growing()
        } else {
            Ring::new(self.window)
        }
    }

    /// Pushes one sample: a value per series, gaps for the fields it lacks.
    fn push(&self, sample: &[f64]) {
        let mut rings = self.rings.lock().expect("rings lock");
        if rings.is_empty() {
            rings.extend((0..sample.len().max(1)).map(|_| self.ring()));
        }
        for (index, ring) in rings.iter().enumerate() {
            ring.push(sample.get(index).copied().unwrap_or(f64::NAN));
        }
    }

    fn snapshot(&self) -> Vec<Vec<f64>> {
        self.rings
            .lock()
            .expect("rings lock")
            .iter()
            .map(Ring::snapshot)
            .collect()
    }
}

/// Builds the live `line` plot from the windows, one line per series, applying
/// the furniture that makes sense for a sliding index axis (title, labels, unit,
/// y limits and log). The x axis is the moving window, so x-domain, x-log, and
/// time-x are deliberately not applied.
fn plot(series: Vec<Vec<f64>>, args: &Args) -> Plot<'static> {
    let mut plot = series
        .into_iter()
        .fold(Plot::new(), |plot, values| plot.layer(Line::y(values)));
    for &value in &args.hlines {
        plot = plot.layer(malevich::Rule::h(value));
    }
    if let Some(unit) = &args.unit {
        plot = plot.y_unit(unit.clone());
    }
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

/// Spawns the reader: the numeric fields of each input line into `rings`,
/// forever. Returns a flag it raises at EOF so the render loop can draw a final
/// frame and stop.
fn spawn_reader(rings: Rings, delimiter: Option<char>, rate: bool) -> Arc<AtomicBool> {
    let done = Arc::new(AtomicBool::new(false));
    let eof = done.clone();
    thread::spawn(move || {
        let stdin = io::stdin();
        let mut trackers: Vec<Rate> = Vec::new();
        for line in stdin.lock().lines() {
            let Ok(line) = line else { break };
            if line.trim().is_empty() {
                continue;
            }
            // A line without numbers is a missed sample — an honest gap, not
            // a value; so is a field a series lacks.
            let mut sample = numbers(&line, delimiter);
            if sample.is_empty() {
                sample.push(f64::NAN);
            }
            if rate {
                if trackers.len() < sample.len() {
                    trackers.resize_with(sample.len(), Rate::new);
                }
                for (value, tracker) in sample.iter_mut().zip(&mut trackers) {
                    *value = tracker.delta(*value);
                }
            }
            rings.push(&sample);
        }
        eof.store(true, Ordering::Relaxed);
    });
    done
}

/// Every field of `line` that parses as a finite number, in order.
fn numbers(line: &str, delimiter: Option<char>) -> Vec<f64> {
    let fields: Box<dyn Iterator<Item = &str>> = match delimiter {
        Some(sep) => Box::new(line.split(sep)),
        None => Box::new(line.split_whitespace()),
    };
    fields
        .filter_map(|field| field.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite())
        .collect()
}

/// The first field of `line` that parses as a finite number.
#[cfg(test)]
fn first_number(line: &str, delimiter: Option<char>) -> Option<f64> {
    numbers(line, delimiter).first().copied()
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
            libc::signal(
                libc::SIGINT,
                on_interrupt as *const () as usize as libc::sighandler_t,
            );
        }
    }
}

#[cfg(test)]
#[path = "tests/live_tests.rs"]
mod tests;
