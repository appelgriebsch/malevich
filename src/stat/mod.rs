//! Statistical transforms: aggregation that runs before scales see the data.
//!
//! Every aggregator here is a mergeable monoid — two partial results combine
//! associatively via `merge` — so host-side parallel chunking (over a fixed
//! reduction tree) and streaming increments are compositions, not features. The
//! plot pipeline inserts [`m4`] automatically for large line layers; everything is
//! also public API for direct use.

mod agg;
mod bin;
mod box_stats;
mod contour;
mod ecdf;
mod fit;
mod kde;
mod lttb;
mod m4;
mod moments;
mod reducer;
mod stack;
mod window;

/// Defensive ceiling for caller-selected statistical output geometry.
pub(crate) const MAX_STAT_ELEMENTS: usize = 1_000_000;

pub use agg::Agg;
pub use bin::{Bins, Histogram2d, binned, bins2, try_bins2};
pub use box_stats::BoxStats;
pub use contour::{Contour, contours};
pub use ecdf::ecdf;
pub use fit::Fit;
pub use kde::kde;
pub use lttb::lttb;
pub(crate) use m4::m4_mapped;
pub use m4::{M4, m4};
pub use moments::Moments;
pub use reducer::{Reducer, quantiles};
pub use stack::stack;
pub use window::Window;
