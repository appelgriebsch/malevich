//! Hybrid pixel rendering: text chrome woven around an image panel.
//!
//! The output is one string in three movements: the full text grid (chrome as
//! always, panel cells blank), a cursor walk to the panel origin, the image, and
//! a walk back. Cursor movement is bracketed by DECSC/DECRC — where a terminal
//! leaves the cursor after an image varies by protocol and emulator, and
//! restoring the saved position sidesteps the whole question. Columns are
//! addressed absolutely (CHA), rows only ever relatively — so the block prints
//! correctly at any *row*, and `column` anchors it horizontally: every text row
//! and the cursor walk jump there first, which is what lets a host paste a
//! pixel plot beside other content (cells left, pixels right).

use std::fmt::Write as _;

use super::{Graphics, PixelCanvas, Protocol, iterm, kitty, sixel};
use crate::plot::{Frame, Plot};
use crate::render::PlotRect;

pub(crate) fn render(plot: &Plot<'_>, frame: &Frame, graphics: &Graphics, column: usize) -> String {
    let cell = (graphics.cell_size.0 as usize, graphics.cell_size.1 as usize);
    if cell.0 == 0 || cell.1 == 0 {
        // No pixel geometry to draw into: degrade to ordinary cell output.
        return at_column(&plot.render(frame), column);
    }
    let (surface, canvas, rect) = plot.rasterize_hybrid(frame, cell);
    let mut out = at_column(&surface.encode(frame.color), column);
    if rect.columns == 0 || rect.rows == 0 {
        return out;
    }
    let image = crop(&canvas, rect);
    if image.width == 0 || image.height == 0 {
        return out;
    }
    let payload = match graphics.protocol {
        Protocol::Sixel => sixel::encode(&image),
        Protocol::Kitty => kitty::encode(&image),
        Protocol::ITerm2 => iterm::encode(&image, rect.columns, rect.rows),
    };
    out.push_str("\x1b7");
    // CHA is 1-based; land on the block's column, then walk to the panel.
    let _ = write!(out, "\x1b[{}G", column + 1);
    let up = frame.height - 1 - rect.top;
    if up > 0 {
        let _ = write!(out, "\x1b[{up}A");
    }
    if rect.gutter > 0 {
        let _ = write!(out, "\x1b[{}C", rect.gutter);
    }
    out.push_str(&payload);
    out.push_str("\x1b8");
    out
}

/// Anchors every row of a rendered block at `column`: each row starts with an
/// absolute-column jump (CHA), so printing the block leaves anything to its
/// left untouched. Column 0 stays escape-free — flush-left output is plain.
fn at_column(text: &str, column: usize) -> String {
    if column == 0 {
        return text.to_string();
    }
    let jump = format!("\x1b[{}G", column + 1);
    let mut out = String::with_capacity(text.len() + jump.len() * 32);
    for (index, row) in text.split('\n').enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&jump);
        out.push_str(row);
    }
    out
}

/// A concrete color, resolved for pixel output.
pub(crate) type Rgb = (u8, u8, u8);

/// A row-major RGB image with transparency: `None` pixels stay undrawn, so the
/// terminal background shows through.
pub(crate) struct Image {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Option<Rgb>>,
}

/// The panel rectangle of the canvas as an [`Image`] of resolved RGB pixels.
fn crop(canvas: &PixelCanvas, rect: PlotRect) -> Image {
    let (cw, ch) = canvas.cell();
    let (x0, y0) = (rect.gutter * cw, rect.top * ch);
    let (width, height) = (rect.columns * cw, rect.rows * ch);
    let mut pixels = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            pixels.push(canvas.get(x0 + x, y0 + y).map(|color| color.to_rgb()));
        }
    }
    Image {
        width,
        height,
        pixels,
    }
}

#[cfg(test)]
#[path = "tests/render_tests.rs"]
mod tests;
