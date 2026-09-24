//! Kernel density estimation: a smooth distribution from a sample.

/// How a KDE chooses its bandwidth.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[non_exhaustive]
pub enum Bandwidth {
    /// Silverman's rule of thumb, `0.9 · min(σ, IQR / 1.34) · n^(−1/5)` — the
    /// default, and R's `bw.nrd0`.
    #[default]
    Silverman,
    /// Silverman's bandwidth times a factor: `Scale(0.5)` halves the smoothing
    /// (seaborn's `bw_adjust`).
    Scale(f64),
    /// A bandwidth in the data's own units.
    Fixed(f64),
}

/// Configuration for [`kde_with`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct KdeOptions {
    /// The bandwidth rule.
    pub bandwidth: Bandwidth,
    /// The support's lower and upper bounds, either optional. A bounded
    /// density reflects its kernels at the bound, so a latency distribution
    /// never leaks mass below zero and a fraction stays inside `[0, 1]`;
    /// values outside the bounds are not part of the sample.
    pub bounds: (Option<f64>, Option<f64>),
    /// How many bandwidths the evaluation grid extends past the data on each
    /// unbounded side; `3.0` by default.
    pub cut: f64,
    /// Evaluate the cumulative density instead: the integral from the grid's
    /// start, rising toward 1.
    pub cumulative: bool,
}

impl KdeOptions {
    /// Silverman's bandwidth, unbounded, three bandwidths of padding, a
    /// density — exactly [`kde`].
    pub const fn new() -> KdeOptions {
        KdeOptions {
            bandwidth: Bandwidth::Silverman,
            bounds: (None, None),
            cut: 3.0,
            cumulative: false,
        }
    }

    /// Sets the bandwidth rule.
    #[must_use]
    pub const fn bandwidth(mut self, bandwidth: Bandwidth) -> KdeOptions {
        self.bandwidth = bandwidth;
        self
    }

    /// Bounds the support; `None` on a side leaves it open.
    #[must_use]
    pub const fn bounds(mut self, low: Option<f64>, high: Option<f64>) -> KdeOptions {
        self.bounds = (low, high);
        self
    }

    /// Sets the padding past the data, in bandwidths.
    #[must_use]
    pub const fn cut(mut self, bandwidths: f64) -> KdeOptions {
        self.cut = bandwidths;
        self
    }

    /// Evaluates the cumulative density.
    #[must_use]
    pub const fn cumulative(mut self) -> KdeOptions {
        self.cumulative = true;
        self
    }
}

impl Default for KdeOptions {
    fn default() -> KdeOptions {
        KdeOptions::new()
    }
}

/// The Gaussian KDE of `values`, evaluated at `points` positions across the data
/// extent (padded by three bandwidths). Returns `(positions, densities)`, or `None`
/// without finite values.
///
/// Bandwidth follows Silverman's rule of thumb —
/// `0.9 * min(σ, IQR / 1.34) * n^(-1/5)` — and evaluation runs on linearly binned
/// counts with a truncated Gaussian kernel: O(n + points × kernel), no FFT.
/// Returns `None` when `points` exceeds the defensive statistics budget.
/// [`kde_with`] chooses the bandwidth, bounds the support, or accumulates.
pub fn kde(values: &[f64], points: usize) -> Option<(Vec<f64>, Vec<f64>)> {
    kde_with(values, points, KdeOptions::new()).ok().flatten()
}

/// [`kde`] configured by [`KdeOptions`]. `Ok(None)` without finite values in
/// the support.
///
/// # Errors
///
/// Returns an error for a bandwidth or factor that is not finite and
/// positive, a negative or non-finite `cut`, non-finite or reversed bounds,
/// or a `points` count beyond the defensive statistics budget.
pub fn kde_with(
    values: &[f64],
    points: usize,
    options: KdeOptions,
) -> crate::Result<Option<(Vec<f64>, Vec<f64>)>> {
    if points > super::MAX_STAT_ELEMENTS {
        return Err(crate::Error::DimensionTooLarge {
            what: "kde points",
            requested: points,
            limit: super::MAX_STAT_ELEMENTS,
        });
    }
    match options.bandwidth {
        Bandwidth::Scale(factor) | Bandwidth::Fixed(factor)
            if !(factor.is_finite() && factor > 0.0) =>
        {
            return Err(crate::Error::InvalidParameter {
                detail: "a kde bandwidth or factor must be finite and positive",
            });
        }
        _ => {}
    }
    if !(options.cut.is_finite() && options.cut >= 0.0) {
        return Err(crate::Error::InvalidParameter {
            detail: "a kde cut must be finite and non-negative",
        });
    }
    let (bound_low, bound_high) = options.bounds;
    if bound_low.is_some_and(|b| !b.is_finite()) || bound_high.is_some_and(|b| !b.is_finite()) {
        return Err(crate::Error::InvalidParameter {
            detail: "kde bounds must be finite",
        });
    }
    if let (Some(low), Some(high)) = (bound_low, bound_high)
        && low >= high
    {
        return Err(crate::Error::InvalidParameter {
            detail: "kde bounds must be ascending",
        });
    }

    let mut finite = Vec::with_capacity(values.len());
    let mut moments = super::Moments::new();
    for &value in values {
        let inside = bound_low.is_none_or(|b| value >= b) && bound_high.is_none_or(|b| value <= b);
        if value.is_finite() && inside {
            moments.add(value);
            finite.push(value);
        }
    }
    if finite.is_empty() || points < 2 {
        return Ok(None);
    }
    let n = moments.count() as f64;
    let Some(sigma) = moments.standard_deviation() else {
        return Ok(None);
    };

    finite.sort_by(f64::total_cmp);
    let quantile = |p: f64| super::reducer::quantile_sorted(&finite, p);
    let (q1, q3) = (quantile(0.25), quantile(0.75));
    let iqr = if q1 < q3 {
        crate::numeric::span_per(q1, q3, 1).unwrap_or(f64::INFINITY)
    } else {
        0.0
    };
    let spread = if iqr > 0.0 {
        sigma.min(iqr / 1.34)
    } else {
        sigma
    };
    let silverman = if spread > 0.0 {
        0.9 * spread * n.powf(-0.2)
    } else {
        // A degenerate sample still deserves a bump rather than a spike.
        1.0
    };
    let bandwidth = match options.bandwidth {
        Bandwidth::Silverman => silverman,
        Bandwidth::Scale(factor) => silverman * factor,
        Bandwidth::Fixed(width) => width,
    };

    let (low, high) = (finite[0], finite[finite.len() - 1]);
    let mut start = low - options.cut * bandwidth;
    let mut end = high + options.cut * bandwidth;
    // The grid stops at a bound, and the kernels reflect there.
    let reflect_low = bound_low.is_some_and(|bound| {
        let clipped = start <= bound;
        if clipped {
            start = bound;
        }
        clipped
    });
    let reflect_high = bound_high.is_some_and(|bound| {
        let clipped = end >= bound;
        if clipped {
            end = bound;
        }
        clipped
    });
    if end <= start || !(end.is_finite() && start.is_finite()) {
        return Ok(None);
    }
    let Some(step) = crate::numeric::span_per(start, end, points - 1) else {
        return Ok(None);
    };
    // At extreme magnitudes the padding can fall below the value's ULP, so the
    // grid collapses (step 0 or non-finite). A single point has no density
    // curve; refuse rather than binning through a zero step into a giant
    // allocation.
    if !(step.is_finite() && step > 0.0) {
        return Ok(None);
    }

    // Linear binning onto the evaluation grid.
    let mut binned = vec![0.0f64; points];
    for &value in &finite {
        let position = crate::numeric::inverse_lerp(start, end, value) * (points - 1) as f64;
        let index = position.floor() as usize;
        let fraction = position - position.floor();
        if index + 1 < points {
            binned[index] += 1.0 - fraction;
            binned[index + 1] += fraction;
        } else {
            binned[points - 1] += 1.0;
        }
    }

    // Truncated Gaussian kernel over the binned counts, never wider than the grid.
    let radius = ((3.0 * bandwidth / step).ceil() as usize).clamp(1, points);
    let kernel: Vec<f64> = (0..=radius)
        .map(|k| {
            let distance = k as f64 * step / bandwidth;
            (-0.5 * distance * distance).exp()
        })
        .collect();
    let normalization = 1.0 / (n * bandwidth * (2.0 * std::f64::consts::PI).sqrt());

    let mut densities: Vec<f64> = (0..points)
        .map(|i| {
            let mut sum = binned[i] * kernel[0];
            for k in 1..=radius {
                if i >= k {
                    sum += binned[i - k] * kernel[k];
                }
                if i + k < points {
                    sum += binned[i + k] * kernel[k];
                }
            }
            // Reflection at a reached bound: the mass a kernel would spill
            // past the grid's edge folds back onto it. A bin `k` steps in from
            // the edge reflects onto the position `i` at distance `i + k`
            // (low end) or `(last − i) + (last − k)` (high end).
            if reflect_low {
                for k in 0..=radius.saturating_sub(i) {
                    if k < points && i + k <= radius {
                        sum += binned[k] * kernel[i + k];
                    }
                }
            }
            if reflect_high {
                let last = points - 1;
                for k in 0..=radius.saturating_sub(last - i) {
                    if k <= last && (last - i) + k <= radius {
                        sum += binned[last - k] * kernel[(last - i) + k];
                    }
                }
            }
            sum * normalization
        })
        .collect();
    if options.cumulative {
        // The trapezoid integral from the grid's start: zero there, rising
        // toward one (minus the kernel's truncated tails).
        let mut total = 0.0;
        let mut previous = densities[0];
        densities[0] = 0.0;
        for density in densities.iter_mut().skip(1) {
            total += 0.5 * (previous + *density) * step;
            previous = *density;
            *density = total;
        }
    }
    let positions = (0..points)
        .map(|index| crate::numeric::lerp(start, end, index as f64 / (points - 1) as f64))
        .collect();
    Ok(Some((positions, densities)))
}

#[cfg(test)]
#[path = "tests/kde_tests.rs"]
mod tests;
