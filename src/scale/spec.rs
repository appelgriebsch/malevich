//! The scale specification: what an axis means, as a value.

/// The scale of one axis.
///
/// Set with [`crate::Plot::x_scale`] / [`crate::Plot::y_scale`]; the sugar methods
/// (`log_y()`, `time_x()`) are shorthands for the common cases. [`Auto`](Scale::Auto)
/// is the default and adapts to the layers; an explicit scale is always honored.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Scale {
    /// Chosen from the layers: categorical when a bars or band-range layer is present,
    /// linear otherwise. The default — and the only scale that infers, so setting any
    /// other value is respected even when a categorical layer is also present.
    #[default]
    Auto,
    /// A continuous linear axis.
    Linear,
    /// Base-10 logarithmic: decade ticks, and values at or below zero become gaps.
    Log,
    /// Unix seconds (UTC): calendar-aligned ticks with multi-scale labels.
    Time,
    /// Named bands — the categorical axis of bar charts, box plots, and violins.
    /// Continuous layers position x against band indices (0 is the first band's
    /// center). Only supported on the x axis.
    Bands(Vec<String>),
}

impl Scale {
    /// Bands from anything yielding names — sugar for [`Scale::Bands`].
    pub fn bands(categories: impl IntoIterator<Item = impl Into<String>>) -> Scale {
        Scale::Bands(categories.into_iter().map(Into::into).collect())
    }
}
