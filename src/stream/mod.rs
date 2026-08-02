//! Streaming: a concurrent sliding window and a flicker-free repaint handle.
//!
//! The [`Ring`] is the one lock in the library (see the design notes on
//! concurrency): producer threads `push` while a render thread takes cheap
//! snapshots. The [`Live`] handle repaints a chart in place — cursor up, erase
//! down, redraw — in a single buffered write, so scrollback survives and nothing
//! flickers. [`Rate`] turns monotonic counters into per-push deltas, the shape live
//! charts usually want.
//!
//! ```no_run
//! use malevich::stream::{Live, Ring};
//!
//! let ring = Ring::new(120);
//! let producer = ring.clone();
//! std::thread::spawn(move || {
//!     loop {
//!         producer.push(read_some_metric());
//!         std::thread::sleep(std::time::Duration::from_millis(100));
//!     }
//! });
//!
//! let mut live = Live::new(std::io::stderr());
//! loop {
//!     let chart = malevich::line(ring.snapshot());
//!     live.draw(&chart, &malevich::Frame::detect()).unwrap();
//!     std::thread::sleep(std::time::Duration::from_millis(250));
//! }
//! # fn read_some_metric() -> f64 { 0.0 }
//! ```

use std::collections::VecDeque;
use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::plot::{Frame, Plot};

/// A sliding window of the most recent values, shared across threads.
///
/// Cloning shares the same window. Pushing past `capacity` drops the oldest value.
#[derive(Debug, Clone)]
pub struct Ring {
    inner: Arc<Mutex<VecDeque<f64>>>,
    capacity: usize,
}

impl Ring {
    /// An empty window holding at most `capacity` values.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is zero.
    pub fn new(capacity: usize) -> Ring {
        assert!(capacity > 0, "Ring::new requires a non-zero capacity");
        Ring {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
        }
    }

    /// Appends a value, dropping the oldest past capacity. Gaps (`NaN`) are values
    /// too — a missed sample stays a visible break.
    pub fn push(&self, value: f64) {
        let mut window = self.inner.lock().expect("ring lock");
        if window.len() == self.capacity {
            window.pop_front();
        }
        window.push_back(value);
    }

    /// The current window, oldest first.
    pub fn snapshot(&self) -> Vec<f64> {
        let window = self.inner.lock().expect("ring lock");
        window.iter().copied().collect()
    }

    /// The number of values currently held.
    pub fn len(&self) -> usize {
        self.inner.lock().expect("ring lock").len()
    }

    /// Whether the window is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Turns a monotonic counter into per-push deltas — bytes into bytes-per-interval.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rate {
    previous: Option<f64>,
}

impl Rate {
    /// A rate tracker with no history.
    pub fn new() -> Rate {
        Rate::default()
    }

    /// The change since the previous call — a gap for the first call (there is
    /// nothing honest to report yet) and after any non-finite reading.
    pub fn delta(&mut self, value: f64) -> f64 {
        let delta = match (self.previous, value.is_finite()) {
            (Some(previous), true) => value - previous,
            _ => f64::NAN,
        };
        self.previous = value.is_finite().then_some(value);
        delta
    }
}

/// Repaints a chart in place: cursor up, erase down, redraw — one buffered write.
///
/// The first draw simply prints. Later draws move the cursor up over the previous
/// frame and erase downward before writing, so the chart updates in place without
/// flicker, survives in scrollback, and never takes over the screen. Query the
/// frame each draw ([`Frame::detect`]) and resizes follow along.
#[derive(Debug)]
pub struct Live<W: Write> {
    out: W,
    drawn_rows: usize,
}

impl<W: Write> Live<W> {
    /// A repaint handle writing to `out` (commonly stderr, leaving stdout to data).
    pub fn new(out: W) -> Live<W> {
        Live { out, drawn_rows: 0 }
    }

    /// Renders the plot and repaints it over the previous frame.
    pub fn draw(&mut self, plot: &Plot<'_>, frame: &Frame) -> std::io::Result<()> {
        let text = plot.render(frame);
        let mut buffer = String::with_capacity(text.len() + 16);
        if self.drawn_rows > 0 {
            use std::fmt::Write as _;
            let _ = write!(buffer, "\x1b[{}A\r\x1b[J", self.drawn_rows);
        }
        buffer.push_str(&text);
        buffer.push('\n');
        self.out.write_all(buffer.as_bytes())?;
        self.out.flush()?;
        self.drawn_rows = text.lines().count().max(1);
        Ok(())
    }

    /// Stops repainting: the next draw starts fresh below the current output.
    pub fn detach(&mut self) {
        self.drawn_rows = 0;
    }
}

#[cfg(test)]
#[path = "tests/stream_tests.rs"]
mod tests;
