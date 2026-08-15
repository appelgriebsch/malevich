//! Streaming ordinary least squares: slope, intercept, and R² in one pass.

/// A least-squares line fit over a stream of `(x, y)` pairs: bivariate
/// Welford accumulation of the means, variances, and covariance, from which
/// the slope, intercept, coefficient of determination, and the standard error
/// of the mean response derive.
///
/// A mergeable monoid: partial results over chunks combine with
/// [`Fit::merge`] (Chan's parallel formula) into the same statistics a single
/// pass produces — host-side parallelism and streaming are compositions.
/// Pairs with a non-finite member are ignored, matching the gap convention.
#[derive(Debug, Clone, Copy)]
pub struct Fit {
    count: u64,
    mean_x: f64,
    mean_y: f64,
    /// Σ(x−x̄)² — the x sum of squares.
    m2_x: f64,
    /// Σ(y−ȳ)² — the y sum of squares.
    m2_y: f64,
    /// Σ(x−x̄)(y−ȳ) — the co-moment.
    co: f64,
}

impl Fit {
    /// An empty accumulator.
    pub fn new() -> Fit {
        Fit {
            count: 0,
            mean_x: 0.0,
            mean_y: 0.0,
            m2_x: 0.0,
            m2_y: 0.0,
            co: 0.0,
        }
    }

    /// The fit of paired slices — [`Fit::add`] over `x.iter().zip(y)`.
    ///
    /// # Panics
    ///
    /// Panics if the slices have different lengths.
    pub fn xy(x: &[f64], y: &[f64]) -> Fit {
        assert_eq!(x.len(), y.len(), "Fit::xy requires slices of equal length");
        let mut fit = Fit::new();
        for (&x, &y) in x.iter().zip(y) {
            fit.add(x, y);
        }
        fit
    }

    /// Accumulates one pair; pairs with a non-finite member are ignored.
    pub fn add(&mut self, x: f64, y: f64) {
        if !x.is_finite() || !y.is_finite() {
            return;
        }
        self.count += 1;
        let count = self.count as f64;
        let delta_x = x - self.mean_x;
        let delta_y = y - self.mean_y;
        self.mean_x += delta_x / count;
        self.mean_y += delta_y / count;
        // The second factors use the updated means (Welford's trick), keeping
        // every accumulator cancellation-free for offset data like timestamps.
        self.m2_x += delta_x * (x - self.mean_x);
        self.m2_y += delta_y * (y - self.mean_y);
        self.co += delta_x * (y - self.mean_y);
    }

    /// Merges another accumulator into this one.
    pub fn merge(&mut self, other: &Fit) {
        if other.count == 0 {
            return;
        }
        if self.count == 0 {
            *self = *other;
            return;
        }
        let total = self.count + other.count;
        let weight = self.count as f64 * other.count as f64 / total as f64;
        let delta_x = other.mean_x - self.mean_x;
        let delta_y = other.mean_y - self.mean_y;
        self.m2_x += other.m2_x + delta_x * delta_x * weight;
        self.m2_y += other.m2_y + delta_y * delta_y * weight;
        self.co += other.co + delta_x * delta_y * weight;
        self.mean_x += delta_x * other.count as f64 / total as f64;
        self.mean_y += delta_y * other.count as f64 / total as f64;
        self.count = total;
    }

    /// The number of finite pairs seen.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// The least-squares slope, or `None` while the fit is degenerate (fewer
    /// than two pairs, or no x spread).
    pub fn slope(&self) -> Option<f64> {
        (self.count >= 2 && self.m2_x > 0.0).then(|| self.co / self.m2_x)
    }

    /// The least-squares intercept, or `None` while the fit is degenerate.
    pub fn intercept(&self) -> Option<f64> {
        self.slope().map(|slope| self.mean_y - slope * self.mean_x)
    }

    /// The fitted value at `x`, or `None` while the fit is degenerate.
    pub fn predict(&self, x: f64) -> Option<f64> {
        self.slope()
            .map(|slope| self.mean_y + slope * (x - self.mean_x))
    }

    /// The coefficient of determination R², or `None` while the fit is
    /// degenerate. Constant y fits perfectly: R² is 1.
    pub fn r_squared(&self) -> Option<f64> {
        self.slope()?;
        if self.m2_y == 0.0 {
            return Some(1.0);
        }
        Some((self.co * self.co / (self.m2_x * self.m2_y)).clamp(0.0, 1.0))
    }

    /// The standard error of the *mean response* at `x` — the half-width unit
    /// of a confidence band around the fitted line (multiply by a quantile:
    /// 1.96 approximates a 95% band for large samples). `None` while the fit
    /// is degenerate or has no residual degrees of freedom (fewer than three
    /// pairs).
    pub fn standard_error(&self, x: f64) -> Option<f64> {
        self.slope()?;
        if self.count < 3 {
            return None;
        }
        let residual = (self.m2_y - self.co * self.co / self.m2_x).max(0.0);
        let variance = residual / (self.count - 2) as f64;
        let offset = x - self.mean_x;
        let leverage = 1.0 / self.count as f64 + offset * offset / self.m2_x;
        Some((variance * leverage).sqrt())
    }
}

impl Default for Fit {
    /// The empty accumulator — identical to [`Fit::new`].
    fn default() -> Fit {
        Fit::new()
    }
}

#[cfg(test)]
#[path = "tests/fit_tests.rs"]
mod tests;
