//! Steps: the piecewise-constant expansion of a series.

/// Where a step changes between two samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum StepDirection {
    /// Hold each value until the next sample: the change happens at the
    /// next x (the default; counters, states, `stairs`).
    #[default]
    Post,
    /// Take each value from the previous sample on: the change happens at
    /// the previous x.
    Pre,
    /// Change halfway between samples.
    Mid,
}

/// The step expansion of `(x, y)`: the polyline that holds each value flat
/// and changes it at the next sample, the previous one, or the midpoint.
/// Feeds [`Line::xy`](crate::Line::xy) — or [`Area::xy`](crate::Area::xy),
/// for a filled step chart. A gap (`NaN`) in either channel breaks the
/// steps there, never bridged.
///
/// ```
/// use malevich::stat::{StepDirection, steps};
///
/// let (x, y) = steps(&[0.0, 1.0, 2.0], &[3.0, 5.0, 4.0], StepDirection::Post);
/// assert_eq!(x, [0.0, 1.0, 1.0, 2.0, 2.0]);
/// assert_eq!(y, [3.0, 3.0, 5.0, 5.0, 4.0]);
/// ```
///
/// # Panics
///
/// Panics if the two slices have different lengths.
pub fn steps(x: &[f64], y: &[f64], direction: StepDirection) -> (Vec<f64>, Vec<f64>) {
    assert_eq!(x.len(), y.len(), "steps requires slices of equal length");
    let mut xs = Vec::with_capacity(x.len() * 2);
    let mut ys = Vec::with_capacity(y.len() * 2);
    let mut previous: Option<(f64, f64)> = None;
    for (&px, &py) in x.iter().zip(y) {
        if !(px.is_finite() && py.is_finite()) {
            // The gap itself, so the polyline breaks here.
            xs.push(f64::NAN);
            ys.push(f64::NAN);
            previous = None;
            continue;
        }
        if let Some((qx, qy)) = previous {
            match direction {
                StepDirection::Post => {
                    xs.push(px);
                    ys.push(qy);
                }
                StepDirection::Pre => {
                    xs.push(qx);
                    ys.push(py);
                }
                StepDirection::Mid => {
                    let middle = crate::numeric::midpoint(qx, px);
                    xs.push(middle);
                    ys.push(qy);
                    xs.push(middle);
                    ys.push(py);
                }
            }
        }
        xs.push(px);
        ys.push(py);
        previous = Some((px, py));
    }
    (xs, ys)
}

#[cfg(test)]
#[path = "tests/steps_tests.rs"]
mod tests;
