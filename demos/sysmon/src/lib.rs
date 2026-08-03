//! A live system monitor built on malevich.
//!
//! The crate splits three ways, mirroring malevich's own spec-then-render
//! philosophy: a data layer ([`data`] — sampling and sliding-window history built
//! on `malevich::stream::Ring`/`Rate`; unit-tested), a pure view layer ([`views`]
//! — snapshots in, `malevich::Plot` out), and a thin binary that owns the
//! terminal and the sampler thread.

pub mod data;
pub mod views;
