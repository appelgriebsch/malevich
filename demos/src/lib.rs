//! Shared library for the malevich demo apps.
//!
//! Each demo splits three ways, mirroring malevich's own spec-then-render
//! philosophy: a pure data layer (parse, transform — unit-tested), a pure view
//! layer (data in, `malevich::Plot` out), and a thin binary that owns the terminal.

pub mod fred;
