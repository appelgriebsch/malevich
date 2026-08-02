//! M4 downsampling: min/max/first/last per raster column.
//!
//! Jugel, Fischer, Mahlmann, Markl, "M4: A Visualization-Oriented Time Series Data
//! Aggregation" (PVLDB 2014): keeping the first, last, minimum, and maximum point of
//! every raster column reproduces a line rendering *exactly* at that raster width —
//! downsampling with zero visual error, in one O(n) pass with O(width) memory.

/// One column's aggregate: the four points that matter, in `(x, y)` pairs.
#[derive(Debug, Clone, Copy)]
struct Bucket {
    first: (f64, f64),
    last: (f64, f64),
    min: (f64, f64),
    max: (f64, f64),
    /// Whether a gap (`NaN`) was seen inside this column.
    gap: bool,
}

/// An M4 aggregator over a fixed x-domain divided into equal columns.
///
/// A mergeable monoid: aggregates built over chunks of a series combine with
/// [`M4::merge`] into exactly the state a single pass would have produced, provided
/// chunks are merged in series order (first/last are scan-order concepts).
#[derive(Debug, Clone)]
pub struct M4 {
    domain: (f64, f64),
    buckets: Vec<Option<Bucket>>,
    /// Gaps seen before any point landed in a bucket still break the line.
    leading_gap: bool,
}

impl M4 {
    /// An empty aggregator over `domain`, one bucket per raster `column`.
    ///
    /// # Panics
    ///
    /// Panics if the domain is not finite or `columns` is zero.
    pub fn new(domain: (f64, f64), columns: usize) -> M4 {
        assert!(
            domain.0.is_finite() && domain.1.is_finite() && columns > 0,
            "M4::new requires a finite domain and at least one column"
        );
        M4 {
            domain,
            buckets: vec![None; columns],
            leading_gap: false,
        }
    }

    /// Accumulates one point. A non-finite `y` records a gap; points with a
    /// non-finite or out-of-domain `x` are ignored.
    pub fn add(&mut self, x: f64, y: f64) {
        if !x.is_finite() {
            return;
        }
        let Some(index) = self.bucket_index(x) else {
            return;
        };
        if !y.is_finite() {
            match &mut self.buckets[index] {
                Some(bucket) => bucket.gap = true,
                None => self.leading_gap = true,
            }
            return;
        }
        let point = (x, y);
        match &mut self.buckets[index] {
            Some(bucket) => {
                bucket.last = point;
                if y < bucket.min.1 {
                    bucket.min = point;
                }
                if y > bucket.max.1 {
                    bucket.max = point;
                }
            }
            None => {
                self.buckets[index] = Some(Bucket {
                    first: point,
                    last: point,
                    min: point,
                    max: point,
                    gap: self.leading_gap,
                });
                self.leading_gap = false;
            }
        }
    }

    /// Merges `later` into `self`, as if `later`'s points had been added after
    /// `self`'s. Both sides must share the domain and column count.
    ///
    /// # Panics
    ///
    /// Panics if the two aggregators have different domains or column counts.
    pub fn merge(&mut self, later: &M4) {
        assert!(
            self.domain == later.domain && self.buckets.len() == later.buckets.len(),
            "M4::merge requires identical domains and column counts"
        );
        for (mine, theirs) in self.buckets.iter_mut().zip(later.buckets.iter()) {
            let Some(theirs) = theirs else { continue };
            match mine {
                Some(bucket) => {
                    bucket.last = theirs.last;
                    if theirs.min.1 < bucket.min.1 {
                        bucket.min = theirs.min;
                    }
                    if theirs.max.1 > bucket.max.1 {
                        bucket.max = theirs.max;
                    }
                    bucket.gap |= theirs.gap;
                }
                None => *mine = Some(*theirs),
            }
        }
        self.leading_gap |= later.leading_gap;
    }

    /// Emits the aggregated series: up to four points per column in x order, with a
    /// gap marker (`NaN`) where a column contained one.
    pub fn emit(self) -> (Vec<f64>, Vec<f64>) {
        let mut x = Vec::with_capacity(self.buckets.len() * 4);
        let mut y = Vec::with_capacity(self.buckets.len() * 4);
        for bucket in self.buckets.into_iter().flatten() {
            if bucket.gap {
                x.push(f64::NAN);
                y.push(f64::NAN);
            }
            let mut points = [bucket.first, bucket.min, bucket.max, bucket.last];
            points.sort_by(|a, b| a.0.total_cmp(&b.0));
            for point in points {
                if x.last() != Some(&point.0) || y.last() != Some(&point.1) {
                    x.push(point.0);
                    y.push(point.1);
                }
            }
        }
        (x, y)
    }

    fn bucket_index(&self, x: f64) -> Option<usize> {
        let (lo, hi) = self.domain;
        if x < lo || x > hi {
            return None;
        }
        if hi == lo {
            return Some(0);
        }
        let position = (x - lo) / (hi - lo) * self.buckets.len() as f64;
        Some((position as usize).min(self.buckets.len() - 1))
    }
}

/// Downsamples an x-sorted series to at most four points per raster column,
/// pixel-exact for line rendering at that width. Convenience over [`M4`].
///
/// Returns `None` when `x` is not sorted ascending (M4 reorders points within
/// columns, which only preserves the drawn line for monotonic x) or when the series
/// has no finite x extent.
pub fn m4(x: &[f64], y: &[f64], columns: usize) -> Option<(Vec<f64>, Vec<f64>)> {
    let columns = columns.max(1);
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    let mut previous = f64::NEG_INFINITY;
    for &value in x {
        if !value.is_finite() {
            continue;
        }
        if value < previous {
            return None;
        }
        previous = value;
        lo = lo.min(value);
        hi = hi.max(value);
    }
    if !lo.is_finite() {
        return None;
    }
    let mut aggregate = M4::new((lo, hi), columns);
    for (&xv, &yv) in x.iter().zip(y.iter()) {
        aggregate.add(xv, yv);
    }
    Some(aggregate.emit())
}

/// [`m4`] for an index-plotted series (`x = 0, 1, 2, …`), without materializing the
/// index column. Indices are always sorted, so this never refuses.
pub(crate) fn m4_indexed(y: &[f64], columns: usize) -> Option<(Vec<f64>, Vec<f64>)> {
    if y.is_empty() {
        return None;
    }
    let mut aggregate = M4::new((0.0, (y.len() - 1) as f64), columns.max(1));
    for (index, &value) in y.iter().enumerate() {
        aggregate.add(index as f64, value);
    }
    Some(aggregate.emit())
}

#[cfg(test)]
#[path = "tests/m4_tests.rs"]
mod tests;
