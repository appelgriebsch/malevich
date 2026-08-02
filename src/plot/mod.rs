//! The plot pipeline: retained descriptions, frames, and rendering.

mod chrome;
mod draw;
pub(crate) mod frame;
mod layout;
#[allow(clippy::module_inception)]
mod plot;
mod resolve;

pub use frame::Frame;
pub use plot::Plot;
