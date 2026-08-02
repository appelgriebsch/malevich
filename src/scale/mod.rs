//! Scales: mappings from data domain to raster range, and their ticks.
//!
//! For now this module holds tick placement (`Ticks`); the scale types themselves land
//! together with the plot pipeline.

mod format;
mod ticks;

pub use ticks::{Tick, Ticks};
