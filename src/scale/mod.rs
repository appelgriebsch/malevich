//! Scales: mappings from data domain to raster range, and their ticks.

mod band;
mod colormap;
pub(crate) mod format;
mod linear;
mod palette;
mod spec;
mod ticks;
pub(crate) mod time;
pub(crate) mod unit;

pub use band::Band;
pub use colormap::Colormap;
pub use format::NumberFormat;
pub use linear::Linear;
pub use palette::Palette;
pub use spec::Scale;
pub(crate) use ticks::offset_base;
pub use ticks::{Tick, TickOptions, Ticks};
pub use unit::Unit;
