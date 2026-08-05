//! Rendering: the subpixel surface, charset codecs, and string encoders.
//!
//! Marks draw on a [`Surface`] in subpixel coordinates (raster convention: origin
//! top-left, y grows downward); a [`Charset`] codec maps each cell's subpixel pattern
//! to one glyph; encoders turn the cell grid into a plain or ANSI string. Nothing in
//! this module touches a terminal, and nothing in it panics: drawing outside the
//! surface clips, non-finite coordinates draw nothing.

mod canvas;
mod charset;
mod color;
#[cfg(feature = "evcxr")]
mod html;
mod surface;
mod width;

#[cfg(feature = "pixel")]
pub(crate) use canvas::trace_line;
pub(crate) use canvas::{Canvas, PlotRect};
pub use charset::Charset;
pub use color::{Color, ColorMode};
#[cfg(feature = "pixel")]
pub(crate) use color::{ansi256_to_rgb, rgb_to_256};
#[cfg(feature = "evcxr")]
pub(crate) use html::{card, mime_bundle};
pub use surface::Surface;
pub(crate) use width::{display_width, fit_width_with};
