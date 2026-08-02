//! `Grid`: small multiples — plots pasted side by side into one string.
//!
//! A grid divides its frame evenly among cells, renders each plot independently,
//! and joins the results. Scales are per-plot (each fits its own data); to share an
//! axis across cells, fix it explicitly with [`crate::Plot::y_domain`] /
//! [`crate::Plot::x_domain`] — sharing is a composition, not a mode.

use unicode_width::UnicodeWidthChar;

use super::frame::Frame;
use super::plot::Plot;

/// A row-major grid of plots rendered as one block of text.
///
/// ```
/// use malevich::{Frame, Grid};
///
/// let grid = Grid::new(2)
///     .with(malevich::line(&[1.0, 3.0, 2.0][..]).title("a"))
///     .with(malevich::line(&[2.0, 1.0, 3.0][..]).title("b"));
/// println!("{}", grid.render(&Frame::plain(80, 12)));
/// ```
#[derive(Debug, Clone)]
pub struct Grid<'a> {
    columns: usize,
    plots: Vec<Plot<'a>>,
}

impl<'a> Grid<'a> {
    /// An empty grid `columns` wide; plots fill rows left to right.
    ///
    /// # Panics
    ///
    /// Panics if `columns` is zero.
    pub fn new(columns: usize) -> Grid<'a> {
        assert!(columns > 0, "Grid::new requires at least one column");
        Grid {
            columns,
            plots: Vec::new(),
        }
    }

    /// Adds the next plot, filling rows left to right.
    #[must_use]
    pub fn with(mut self, plot: Plot<'a>) -> Grid<'a> {
        self.plots.push(plot);
        self
    }

    /// Detaches from any borrowed storage, making the grid `'static`.
    pub fn into_owned(self) -> Grid<'static> {
        Grid {
            columns: self.columns,
            plots: self.plots.into_iter().map(Plot::into_owned).collect(),
        }
    }

    /// Renders the grid into `frame`, dividing it evenly among cells with one
    /// column of separation between neighbors. Empty grids render nothing.
    pub fn render(&self, frame: &Frame) -> String {
        if self.plots.is_empty() || frame.width == 0 || frame.height == 0 {
            return String::new();
        }
        let columns = self.columns.min(self.plots.len());
        let rows = self.plots.len().div_ceil(columns);
        let cell_frame = Frame {
            width: (frame.width.saturating_sub(columns - 1) / columns).max(1),
            height: (frame.height / rows).max(1),
            ..*frame
        };

        let mut lines = Vec::new();
        for row in self.plots.chunks(columns) {
            let cells: Vec<Vec<String>> = row
                .iter()
                .map(|plot| {
                    plot.render(&cell_frame)
                        .lines()
                        .map(str::to_string)
                        .collect()
                })
                .collect();
            let height = cells.iter().map(Vec::len).max().unwrap_or(0);
            for index in 0..height {
                let mut line = String::new();
                for (cell_index, cell) in cells.iter().enumerate() {
                    let content = cell.get(index).map(String::as_str).unwrap_or_default();
                    line.push_str(content);
                    if cell_index + 1 < cells.len() {
                        // Pad to the cell width in display columns, escape-aware.
                        for _ in visible_width(content)..cell_frame.width + 1 {
                            line.push(' ');
                        }
                    }
                }
                lines.push(line);
            }
        }
        lines.join("\n")
    }
}

/// The display width of a rendered line: escape sequences are invisible.
fn visible_width(line: &str) -> usize {
    let mut width = 0;
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // Skip a CSI sequence through its final byte.
            for follow in chars.by_ref() {
                if follow.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        width += c.width().unwrap_or(0);
    }
    width
}

#[cfg(test)]
#[path = "tests/grid_tests.rs"]
mod tests;
