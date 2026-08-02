//! Terminal plotting: a small grammar of marks, honest axes, millions of points.
//!
//! Under construction. The first release ships a line chart with real axes; the
//! vocabulary of the crate is defined in `TERMINOLOGY.md`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod data;
pub mod mark;
pub mod plot;
mod presets;
pub mod render;
pub mod scale;
pub mod stat;
mod theme;

pub use mark::{Area, Bars, Line, Mark, Points, Rule, Text};
pub use plot::{Frame, Plot};
pub use presets::{bar, ecdf, hist, line, scatter, stairs};
pub use render::{Charset, Color, ColorMode};
pub use theme::Theme;
