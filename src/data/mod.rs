//! Data ingestion: the rim where anything series-shaped becomes a `Series`.
//!
//! The core of the crate is monomorphic `f64`; conversions happen exactly once, here.
//! Borrowed `f64` slices are zero-copy; every other scalar type is converted and
//! copied once. `NaN` is the gap encoding and flows through ingestion untouched.

#[cfg(feature = "ndarray")]
mod ndarray;
mod series;

pub use series::{IntoSeries, Series};
