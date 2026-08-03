//! A Federal Reserve economic-data browser built on malevich.
//!
//! The crate splits three ways, mirroring malevich's own spec-then-render
//! philosophy: a pure data layer ([`data`] — parsing, calendar math, transforms;
//! unit-tested), a pure view layer ([`views`] — data in, `malevich::Plot` out),
//! and a thin binary that owns the terminal.

pub mod data;
pub mod views;
