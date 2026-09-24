//! Axis units: what the labels of a linear axis are counted in.

/// The unit an axis's labels carry. A scale option, set with
/// [`Plot::x_unit`](crate::Plot::x_unit) / [`Plot::y_unit`](crate::Plot::y_unit)
/// on a linear or integer axis; the ticks stay the extended-Wilkinson ticks,
/// only the labels change — and, for binary bytes, the unit they are nice in.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Unit {
    /// Bare numbers with the axis's one SI prefix attached (`2.5M`, `100µ`).
    #[default]
    Plain,
    /// One SI prefix per axis followed by the unit, separated by a space:
    /// `2.5 MB`, `100 µs`, `0 kB` — the prefix is the axis's, so every label
    /// reads in the same unit.
    Si(String),
    /// Binary bytes: ticks are chosen nice in the axis's binary unit and read
    /// `512 KiB`, `1.5 GiB` — the axis a memory chart wants, and one a caller
    /// cannot compose from decimal ticks.
    Bytes,
    /// A bare suffix on every label and never an SI prefix: `45%`, `3×`.
    Suffix(String),
}

impl Unit {
    /// An SI unit: `Unit::si("B")` labels `2.5 MB`, `Unit::si("s")` labels
    /// `100 µs`.
    pub fn si(unit: impl Into<String>) -> Unit {
        Unit::Si(unit.into())
    }

    /// A bare suffix: `Unit::suffix("%")` labels `45%`.
    pub fn suffix(suffix: impl Into<String>) -> Unit {
        Unit::Suffix(suffix.into())
    }

    /// Whether this is the plain, unit-less form.
    pub fn is_plain(&self) -> bool {
        *self == Unit::Plain
    }
}

/// The binary prefixes, from bytes up.
pub(crate) const BINARY_UNITS: [&str; 9] =
    ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];

/// The binary prefix power for values up to `max_abs` bytes, and its factor.
pub(crate) fn binary_prefix(max_abs: f64) -> (usize, f64) {
    let power = if max_abs.is_finite() && max_abs >= 1024.0 {
        ((max_abs.log2() / 10.0).floor() as usize).min(BINARY_UNITS.len() - 1)
    } else {
        0
    };
    (power, 1024f64.powi(power as i32))
}
