//! `Grid`: small multiples — plots pasted side by side into one string.
//!
//! A grid divides its frame evenly among cells, renders each plot independently,
//! and joins the results. Scales are per-plot (each fits its own data); to share an
//! axis across cells, fix it explicitly with [`crate::Plot::y_domain`] /
//! [`crate::Plot::x_domain`] — sharing is a composition, not a mode.

use super::frame::Frame;
use super::plot::Plot;
use crate::render::display_width_ansi;

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Grid<'a> {
    columns: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    plots: Vec<Plot<'a>>,
}

#[derive(Debug, Clone, Copy)]
struct Layout {
    columns: usize,
    rows: usize,
    visible_plots: usize,
    cell_width: usize,
    cell_height: usize,
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

    /// Checks the grid and every contained plot without rendering.
    ///
    /// This rejects the zero-column state that deserialization can represent and
    /// applies [`Plot::validate`] to every pane.
    pub fn validate(&self) -> crate::Result<()> {
        if self.columns == 0 {
            return Err(crate::Error::EmptyDimension {
                what: "Grid columns",
            });
        }
        for plot in &self.plots {
            plot.validate()?;
        }
        Ok(())
    }

    /// Renders the grid into `frame`, dividing it evenly among cells with one blank
    /// column between neighbors and one blank row between stacked rows.
    ///
    /// Separators consume the frame budget. At tiny sizes, the grid reduces its
    /// visible rows or columns and omits later plots rather than creating over-wide
    /// zero-size panes. Invalid or oversized input degrades to an empty string; use
    /// [`Grid::try_render`] for a typed error.
    pub fn render(&self, frame: &Frame) -> String {
        self.try_render_unvalidated(frame).unwrap_or_default()
    }

    /// Validates and renders the grid, returning spec, geometry, or allocation
    /// failures as typed errors.
    pub fn try_render(&self, frame: &Frame) -> crate::Result<String> {
        self.validate()?;
        self.try_render_unvalidated(frame)
    }

    fn try_render_unvalidated(&self, frame: &Frame) -> crate::Result<String> {
        crate::render::frame_cells(frame.width, frame.height)?;
        if self.columns == 0 || self.plots.is_empty() || frame.width == 0 || frame.height == 0 {
            return Ok(String::new());
        }
        let Some(layout) = self.layout(frame) else {
            return Ok(String::new());
        };

        let cell_frame = Frame {
            width: layout.cell_width,
            height: layout.cell_height,
            ..*frame
        };

        let mut cells = Vec::new();
        crate::render::reserve_vec(&mut cells, layout.visible_plots, "grid pane strings")?;
        for plot in &self.plots[..layout.visible_plots] {
            cells.push(plot.try_render_unvalidated(&cell_frame)?);
        }

        // Measure the exact composed payload before writing it. Pane encoders are
        // independently bounded, but their ANSI resets and the grid's padding make
        // a frame-cell estimate needlessly loose. One exact fallible reservation
        // lets the append-only composition below remain non-panicking.
        let output_bytes = composed_bytes(&cells, layout)?;
        let mut output = String::new();
        crate::render::reserve_string(&mut output, output_bytes, "grid output bytes")?;
        compose(&mut output, &cells, layout)?;
        debug_assert_eq!(output.len(), output_bytes);
        debug_assert_eq!(layout.rows, cells.chunks(layout.columns).len());
        Ok(output)
    }

    fn layout(&self, frame: &Frame) -> Option<Layout> {
        // Every visible pane gets at least one cell; one-cell separators therefore
        // limit how many panes fit along either axis. Later panes are omitted.
        let column_capacity = frame.width.div_ceil(2);
        let row_capacity = frame.height.div_ceil(2);
        let columns = self.columns.min(self.plots.len()).min(column_capacity);
        if columns == 0 || row_capacity == 0 {
            return None;
        }
        let visible_plots = self.plots.len().min(columns.saturating_mul(row_capacity));
        let rows = visible_plots.div_ceil(columns);
        let cell_width = frame.width.saturating_sub(columns - 1) / columns;
        let cell_height = frame.height.saturating_sub(rows - 1) / rows;
        Some(Layout {
            columns,
            rows,
            visible_plots,
            cell_width,
            cell_height,
        })
    }
}

fn composed_bytes(cells: &[String], layout: Layout) -> crate::Result<usize> {
    let mut bytes = 0usize;
    let mut first_line = true;
    for (grid_row, row) in cells.chunks(layout.columns).enumerate() {
        if grid_row > 0 {
            add_line_bytes(&mut bytes, &mut first_line, 0)?;
        }
        let mut lines = Vec::new();
        crate::render::reserve_vec(&mut lines, row.len(), "grid line iterators")?;
        lines.extend(row.iter().map(|cell| cell.lines()));
        for _ in 0..layout.cell_height {
            let mut line_bytes = 0usize;
            for (cell_index, pane_lines) in lines.iter_mut().enumerate() {
                let content = pane_lines.next().unwrap_or_default();
                let visible = display_width_ansi(content);
                if visible > layout.cell_width {
                    return Err(crate::Error::DimensionTooLarge {
                        what: "grid pane width",
                        requested: visible,
                        limit: layout.cell_width,
                    });
                }
                line_bytes = checked_add(line_bytes, content.len())?;
                if cell_index + 1 < row.len() {
                    line_bytes = checked_add(line_bytes, layout.cell_width - visible + 1)?;
                }
            }
            add_line_bytes(&mut bytes, &mut first_line, line_bytes)?;
        }
    }
    Ok(bytes)
}

fn compose(output: &mut String, cells: &[String], layout: Layout) -> crate::Result<()> {
    let mut first_line = true;
    for (grid_row, row) in cells.chunks(layout.columns).enumerate() {
        if grid_row > 0 {
            start_line(output, &mut first_line);
        }
        let mut lines = Vec::new();
        crate::render::reserve_vec(&mut lines, row.len(), "grid line iterators")?;
        lines.extend(row.iter().map(|cell| cell.lines()));
        for _ in 0..layout.cell_height {
            start_line(output, &mut first_line);
            for (cell_index, pane_lines) in lines.iter_mut().enumerate() {
                let content = pane_lines.next().unwrap_or_default();
                output.push_str(content);
                if cell_index + 1 < row.len() {
                    let padding = layout.cell_width - display_width_ansi(content) + 1;
                    output.extend(std::iter::repeat_n(' ', padding));
                }
            }
        }
    }
    Ok(())
}

fn add_line_bytes(bytes: &mut usize, first: &mut bool, content: usize) -> crate::Result<()> {
    if !*first {
        *bytes = checked_add(*bytes, 1)?;
    }
    *first = false;
    *bytes = checked_add(*bytes, content)?;
    Ok(())
}

fn checked_add(left: usize, right: usize) -> crate::Result<usize> {
    left.checked_add(right)
        .filter(|bytes| *bytes <= crate::render::MAX_OUTPUT_BYTES)
        .ok_or(crate::Error::DimensionTooLarge {
            what: "grid output bytes",
            requested: left.saturating_add(right),
            limit: crate::render::MAX_OUTPUT_BYTES,
        })
}

fn start_line(output: &mut String, first: &mut bool) {
    if !*first {
        output.push('\n');
    }
    *first = false;
}

#[cfg(test)]
#[path = "tests/grid_tests.rs"]
mod tests;
