//! Scales: mappings from data domain to raster range, and their ticks.

mod format;
mod linear;
mod ticks;

pub use linear::Linear;
pub use ticks::{Tick, Ticks};
