//! A manual axis domain: both ends fixed, or one end fixed and the other
//! fitted to the data.

/// The ends of an axis a plot fixes: `min`, `max`, or both. A missing end
/// fits the data as an automatic axis would. Built by
/// [`Plot::x_domain`](crate::Plot::x_domain), [`Plot::x_min`](crate::Plot::x_min),
/// [`Plot::x_max`](crate::Plot::x_max) and their y twins; on the wire a
/// two-ended domain is the `[min, max]` pair it always was, a one-ended one
/// is `{"min": v}` or `{"max": v}`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct Bounds {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl Bounds {
    /// Both ends fixed.
    pub(crate) const fn pair(min: f64, max: f64) -> Bounds {
        Bounds {
            min: Some(min),
            max: Some(max),
        }
    }

    /// Both ends, when both are fixed.
    pub(crate) fn both(&self) -> Option<(f64, f64)> {
        Some((self.min?, self.max?))
    }

    /// Which ends are fixed, `(min, max)`.
    pub(crate) const fn fixed(&self) -> (bool, bool) {
        (self.min.is_some(), self.max.is_some())
    }

    /// The domain over `data`: each fixed end replaces its side of the data
    /// extent, and a free end never crosses a fixed one — a floor above the
    /// data leaves a point domain, clipping everything, rather than an axis
    /// that runs backwards.
    pub(crate) fn apply(&self, data: (f64, f64)) -> (f64, f64) {
        match (self.min, self.max) {
            (Some(min), Some(max)) => (min, max),
            (Some(min), None) => (min, data.1.max(min)),
            (None, Some(max)) => (data.0.min(max), max),
            (None, None) => data,
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Bounds {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        match (self.min, self.max) {
            (Some(min), Some(max)) => (min, max).serialize(serializer),
            (min, max) => {
                let mut sides = serializer.serialize_struct(
                    "Bounds",
                    usize::from(min.is_some()) + usize::from(max.is_some()),
                )?;
                if let Some(min) = min {
                    sides.serialize_field("min", &min)?;
                }
                if let Some(max) = max {
                    sides.serialize_field("max", &max)?;
                }
                sides.end()
            }
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Bounds {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;

        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum Wire {
            Pair(f64, f64),
            Sides {
                #[serde(default)]
                min: Option<f64>,
                #[serde(default)]
                max: Option<f64>,
            },
        }

        match Wire::deserialize(deserializer)? {
            Wire::Pair(min, max) => Ok(Bounds::pair(min, max)),
            Wire::Sides {
                min: None,
                max: None,
            } => Err(D::Error::custom(
                "an axis domain fixes a min, a max, or both",
            )),
            Wire::Sides { min, max } => Ok(Bounds { min, max }),
        }
    }
}
