//! `Series`: one column of scalar data, and the `IntoSeries` ingestion trait.

use std::borrow::Cow;

/// One column of scalar data: contiguous `f64`, where `NaN` is a gap.
///
/// A series either borrows a caller's `&[f64]` (zero-copy) or owns its values (any
/// other input converts and copies exactly once at ingestion). Gaps (`NaN`) are
/// preserved — they render as visible breaks, never interpolated across.
///
/// ```
/// use malevich::data::{IntoSeries, Series};
///
/// let borrowed: Series = (&[1.0, 2.0, f64::NAN, 4.0][..]).into_series();
/// let counted: Series = (0..5).map(|i| i as f64).collect();
/// assert_eq!(borrowed.extent(), Some((1.0, 4.0)));
/// assert_eq!(counted.len(), 5);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Series<'a> {
    values: Cow<'a, [f64]>,
}

impl<'a> Series<'a> {
    /// The values, in order, gaps included.
    pub fn as_slice(&self) -> &[f64] {
        &self.values
    }

    /// Iterates over the values, gaps included.
    pub fn iter(&self) -> impl Iterator<Item = f64> + '_ {
        self.values.iter().copied()
    }

    /// The number of values, gaps included.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the series has no values.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// The `(min, max)` over finite values, or `None` if there are none.
    ///
    /// Gaps (`NaN`) and infinities do not participate: an axis domain must be finite.
    pub fn extent(&self) -> Option<(f64, f64)> {
        let mut extent: Option<(f64, f64)> = None;
        for value in self.iter().filter(|value| value.is_finite()) {
            extent = match extent {
                None => Some((value, value)),
                Some((min, max)) => Some((min.min(value), max.max(value))),
            };
        }
        extent
    }

    /// Detaches from any borrowed storage, making the series `'static`.
    pub fn into_owned(self) -> Series<'static> {
        Series {
            values: Cow::Owned(self.values.into_owned()),
        }
    }
}

impl FromIterator<f64> for Series<'static> {
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> Self {
        Series {
            values: Cow::Owned(iter.into_iter().collect()),
        }
    }
}

impl FromIterator<f32> for Series<'static> {
    fn from_iter<I: IntoIterator<Item = f32>>(iter: I) -> Self {
        Series {
            values: Cow::Owned(iter.into_iter().map(f64::from).collect()),
        }
    }
}

/// Conversion into a [`Series`]: the single ingestion boundary of the crate.
///
/// Borrowed `f64` slices are zero-copy; all other implementations convert and copy
/// once. Implemented for slices, arrays, and vectors of every primitive numeric type
/// (integers wider than 53 bits round to the nearest representable `f64`), and for
/// `Series` itself.
pub trait IntoSeries<'a> {
    /// Converts `self` into a series.
    fn into_series(self) -> Series<'a>;
}

impl<'a> IntoSeries<'a> for Series<'a> {
    fn into_series(self) -> Series<'a> {
        self
    }
}

impl<'a> IntoSeries<'a> for &'a [f64] {
    fn into_series(self) -> Series<'a> {
        Series {
            values: Cow::Borrowed(self),
        }
    }
}

impl<'a> IntoSeries<'a> for &'a Vec<f64> {
    fn into_series(self) -> Series<'a> {
        Series {
            values: Cow::Borrowed(self),
        }
    }
}

impl<'a, const N: usize> IntoSeries<'a> for &'a [f64; N] {
    fn into_series(self) -> Series<'a> {
        Series {
            values: Cow::Borrowed(self),
        }
    }
}

impl<'a> IntoSeries<'a> for Vec<f64> {
    fn into_series(self) -> Series<'a> {
        Series {
            values: Cow::Owned(self),
        }
    }
}

impl<'a, const N: usize> IntoSeries<'a> for [f64; N] {
    fn into_series(self) -> Series<'a> {
        Series {
            values: Cow::Owned(self.to_vec()),
        }
    }
}

/// Implements the converting (copy-once) ingestion for a non-`f64` scalar type.
macro_rules! converting_into_series {
    ($($scalar:ty),*) => {$(
        impl<'a, 'b> IntoSeries<'a> for &'b [$scalar] {
            fn into_series(self) -> Series<'a> {
                Series {
                    values: Cow::Owned(self.iter().map(|&value| value as f64).collect()),
                }
            }
        }

        impl<'a, 'b> IntoSeries<'a> for &'b Vec<$scalar> {
            fn into_series(self) -> Series<'a> {
                self.as_slice().into_series()
            }
        }

        impl<'a> IntoSeries<'a> for Vec<$scalar> {
            fn into_series(self) -> Series<'a> {
                self.as_slice().into_series()
            }
        }

        impl<'a, 'b, const N: usize> IntoSeries<'a> for &'b [$scalar; N] {
            fn into_series(self) -> Series<'a> {
                self.as_slice().into_series()
            }
        }

        impl<'a, const N: usize> IntoSeries<'a> for [$scalar; N] {
            fn into_series(self) -> Series<'a> {
                self.as_slice().into_series()
            }
        }
    )*};
}

converting_into_series!(f32, i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

/// With the `serde` feature, a series encodes as a sequence of optional numbers:
/// gaps (`NaN`) become `None`/`null`, so they survive formats like JSON that
/// cannot carry `NaN`, and decode back to gaps exactly.
#[cfg(feature = "serde")]
impl serde::Serialize for Series<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(
            self.iter()
                .map(|value| if value.is_nan() { None } else { Some(value) }),
        )
    }
}

#[cfg(feature = "serde")]
impl<'de, 'a> serde::Deserialize<'de> for Series<'a> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values: Vec<Option<f64>> = serde::Deserialize::deserialize(deserializer)?;
        Ok(values
            .into_iter()
            .map(|value| value.unwrap_or(f64::NAN))
            .collect())
    }
}

#[cfg(test)]
#[path = "tests/series_tests.rs"]
mod tests;
