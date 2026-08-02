//! Scales: mappings from data domain to raster range, and their ticks.

mod band;
mod format;
mod linear;
mod ticks;

pub use band::Band;
pub use linear::Linear;
pub use ticks::{Tick, Ticks};
