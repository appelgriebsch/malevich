//! Scales: mappings from data domain to raster range, and their ticks.

mod band;
mod colormap;
mod format;
mod linear;
mod spec;
mod ticks;
mod time;

pub use band::Band;
pub use colormap::Colormap;
pub use linear::Linear;
pub use spec::Scale;
pub use ticks::{Tick, Ticks};
