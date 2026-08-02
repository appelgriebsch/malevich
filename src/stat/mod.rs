//! Statistical transforms: aggregation that runs before scales see the data.
//!
//! Every aggregator here is a mergeable monoid — two partial results combine
//! associatively via `merge` — so host-side parallel chunking (over a fixed
//! reduction tree) and streaming increments are compositions, not features. The
//! plot pipeline inserts [`m4`] automatically for large line layers; everything is
//! also public API for direct use.

mod agg;
mod bin;
mod ecdf;
mod lttb;
mod m4;
mod moments;
mod stack;

pub use agg::Agg;
pub use bin::{Bins, Grid, bins2};
pub use ecdf::ecdf;
pub use lttb::lttb;
pub(crate) use m4::m4_indexed;
pub use m4::{M4, m4};
pub use moments::Moments;
pub use stack::stack;
