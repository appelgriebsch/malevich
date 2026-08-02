//! The plot pipeline: retained descriptions, frames, and rendering.

mod frame;
#[allow(clippy::module_inception)]
mod plot;

pub use frame::Frame;
pub use plot::Plot;
