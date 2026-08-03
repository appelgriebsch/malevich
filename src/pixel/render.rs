//! Hybrid pixel rendering: text chrome woven around an image panel.
//!
//! The output is one string in three movements: the full text grid (chrome as
//! always, panel cells blank), a cursor walk to the panel origin, the image, and
//! a walk back. Cursor movement is relative and bracketed by DECSC/DECRC — where
//! a terminal leaves the cursor after an image varies by protocol and emulator,
//! and restoring the saved position sidesteps the whole question.

use std::fmt::Write as _;

use super::{Graphics, PixelCanvas, Protocol, sixel};
use crate::plot::{Frame, Plot};
use crate::render::PlotRect;

pub(crate) fn render(plot: &Plot<'_>, frame: &Frame, graphics: &Graphics) -> String {
    let cell = (graphics.cell_size.0 as usize, graphics.cell_size.1 as usize);
    if cell.0 == 0 || cell.1 == 0 {
        // No pixel geometry to draw into: degrade to ordinary cell output.
        return plot.render(frame);
    }
    let (surface, canvas, rect) = plot.rasterize_hybrid(frame, cell);
    let mut out = surface.encode(frame.color);
    if rect.columns == 0 || rect.rows == 0 {
        return out;
    }
    let image = crop(&canvas, rect);
    if image.width == 0 || image.height == 0 {
        return out;
    }
    let payload = match graphics.protocol {
        Protocol::Sixel => sixel::encode(&image),
    };
    out.push_str("\x1b7\r");
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
