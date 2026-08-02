//! Rendering: the subpixel surface, charset codecs, and string encoders.
//!
//! Marks draw on a [`Surface`] in subpixel coordinates (raster convention: origin
//! top-left, y grows downward); a [`Charset`] codec maps each cell's subpixel pattern
//! to one glyph; encoders turn the cell grid into a plain or ANSI string. Nothing in
//! this module touches a terminal, and nothing in it panics: drawing outside the
//! surface clips, non-finite coordinates draw nothing.

mod charset;
mod color;
mod surface;
mod width;

pub use charset::Charset;
pub use color::{Color, ColorMode};
pub use surface::Surface;
pub(crate) use width::{display_width, fit_width};
