//! Terminal plotting: a small grammar of marks, honest axes, millions of points.
//!
//! Eight marks ([`Line`], [`Points`], [`Bars`], [`Area`], [`Cells`], [`Range`],
//! [`Rule`], [`Text`]) compose over shared scales into the basic chart catalog;
//! presets like [`line()`], [`hist`], [`box_plot`], and [`violin`] are one-line fronts
//! over that grammar. Large series aggregate to the raster before drawing (M4 —
//! pixel-exact for lines), axes use extended-Wilkinson tick placement with
//! exact-decimal labels, and everything renders to a plain `String` — colored for
//! your terminal via [`Frame::detect`], deterministic via [`Frame::plain`].
//!
//! ```
//! use malevich::{Frame, Line, Plot, Rule};
//!
//! let steps: Vec<f64> = (0..100).map(f64::from).collect();
//! let loss: Vec<f64> = steps.iter().map(|s| 4.0 * (-0.05 * s).exp() + 0.4).collect();
//! let chart = Plot::new()
//!     .layer(Line::xy(&steps[..], &loss[..]).label("loss"))
//!     .layer(Rule::h(0.5).label("target"))
//!     .title("training");
//! println!("{}", chart.render(&Frame::plain(60, 14)));
//! ```
//!
//! A plot is a plain value: `Clone + Send + Sync`, no global state, rendering is a
//! pure function of plot and frame. The modules follow the concepts (each defined in
//! the repository's `TERMINOLOGY.md`): [`mark`] for the primitives, [`stat`] for
//! transforms (binning, KDE, rolling windows, downsampling — all mergeable),
//! [`scale`] for ticks and colormaps, [`render`] for the subpixel surface and
//! charsets, [`stream`] for live charts, [`data`] for the ingestion rim.
//!
//! The gallery in `EXAMPLES.md` shows every chart type with its source, and
//! `cargo run --example showcase` renders a colored tour in your terminal.
//!
//! # Features
//!
//! - `ndarray` — one-dimensional arrays and views plot directly; contiguous
//!   storage is zero-copy.
//! - `ratatui` — [`PlotWidget`], a `ratatui` widget rendering any plot into a
//!   `Buffer`.
//! - `serde` — every spec type (plots, marks, scales, themes, frames)
//!   round-trips through serde; gaps survive JSON as `null`, and function-backed
//!   lines refuse to serialize rather than lie.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "ratatui")]
mod adapter;
pub mod data;
pub mod mark;
pub mod plot;
mod presets;
pub mod render;
pub mod scale;
#[cfg(all(test, feature = "serde"))]
mod serde_tests;
pub mod stat;
pub mod stream;
mod theme;

#[cfg(feature = "ratatui")]
pub use adapter::PlotWidget;
pub use mark::{Area, Bars, Cells, Line, LineStyle, Mark, Points, Range, Rule, Text};
pub use plot::{Frame, Grid, Plot};
pub use presets::{
    bar, box_plot, contour, density, ecdf, error_bars, heatmap, hist, hist2d, line, quiver,
    scatter, stairs, violin,
};
pub use render::{Charset, Color, ColorMode};
pub use scale::Scale;
pub use theme::Theme;
