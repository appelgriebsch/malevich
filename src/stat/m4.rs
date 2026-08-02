//! M4 downsampling: min/max/first/last per raster column.
//!
//! Jugel, Fischer, Mahlmann, Markl, "M4: A Visualization-Oriented Time Series Data
//! Aggregation" (PVLDB 2014): keeping the first, last, minimum, and maximum point of
//! every raster column reproduces that column's pixels exactly. The plot pipeline
//! buckets by the rendered column ([`m4_mapped`]), so the auto-inserted reduction is
//! pixel-identical to drawing every point — zero visual error, one O(n) pass, O(width)
//! memory.

/// One column's aggregate: the four points that matter, in `(x, y)` pairs.
#[derive(Debug, Clone, Copy)]
struct Bucket {
    first: (f64, f64),
    last: (f64, f64),
    min: (f64, f64),
    max: (f64, f64),
    /// If a gap (`NaN`) fell inside this column, the x of the last finite point
    /// before it — or `-inf` when the gap preceded every finite point here. On
    /// emit the break goes between the points at or before this x and those after,
    /// so a gap never reconnects the values it separated.
    gap: Option<f64>,
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

    /// An aggregator addressed by explicit bucket index instead of by domain — the
    /// bucket count is `columns` and [`M4::record`] places points directly. Used by
    /// [`m4_mapped`], which buckets by the rendered raster column.
    pub(crate) fn columns(columns: usize) -> M4 {
        M4 {
            domain: (0.0, 1.0),
            buckets: vec![None; columns.max(1)],
            leading_gap: false,
        }
    }

    /// Accumulates one point. A non-finite `y` records a gap; points with a
    /// non-finite or out-of-domain `x` are ignored.
    pub fn add(&mut self, x: f64, y: f64) {
        if !x.is_finite() {
            return;
        }
        if let Some(index) = self.bucket_index(x) {
            self.record(index, x, y);
        }
    }

    /// Records `(x, y)` into bucket `index`. A non-finite `y` marks a gap there.
    fn record(&mut self, index: usize, x: f64, y: f64) {
        if !y.is_finite() {
            match &mut self.buckets[index] {
                Some(bucket) => bucket.gap = Some(bucket.last.0),
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
                    gap: self.leading_gap.then_some(f64::NEG_INFINITY),
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
                    // Their points follow mine, so a gap in their column sits at
                    // their x; keep it, else preserve mine.
                    bucket.gap = theirs.gap.or(bucket.gap);
                }
                None => *mine = Some(*theirs),
            }
        }
        self.leading_gap |= later.leading_gap;
    }

    /// Emits the aggregated series: up to four points per column in x order, with a
    /// gap marker (`NaN`) where a column contained one.
    pub fn emit(self) -> (Vec<f64>, Vec<f64>) {
        // Append a point unless it duplicates the last one written (collapses the
        // repeated first/min/max/last of a flat column into one).
        fn push(x: &mut Vec<f64>, y: &mut Vec<f64>, point: (f64, f64)) {
            if x.last() != Some(&point.0) || y.last() != Some(&point.1) {
                x.push(point.0);
                y.push(point.1);
            }
        }
        let mut x = Vec::with_capacity(self.buckets.len() * 4);
        let mut y = Vec::with_capacity(self.buckets.len() * 4);
        for bucket in self.buckets.into_iter().flatten() {
            let mut points = [bucket.first, bucket.min, bucket.max, bucket.last];
            points.sort_by(|a, b| a.0.total_cmp(&b.0));
            match bucket.gap {
                None => {
                    for point in points {
                        push(&mut x, &mut y, point);
                    }
                }
                Some(at) => {
                    // Emit the points that precede the gap, break, then the rest —
                    // so the line is cut exactly where the data was, not before it.
                    for point in points.iter().filter(|p| p.0 <= at) {
                        push(&mut x, &mut y, *point);
                    }
                    x.push(f64::NAN);
                    y.push(f64::NAN);
                    for point in points.iter().filter(|p| p.0 > at) {
                        push(&mut x, &mut y, *point);
                    }
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
/// preserving each column's silhouette. Rendered over the same domain into a raster
/// `columns` wide, the reduction is pixel-exact. Convenience over [`M4`].
///
/// Returns `None` when `x` is not sorted ascending (M4 reorders points within
/// columns, which only preserves the drawn line for monotonic x) or when the series
/// has no finite x extent.
///
/// # Panics
///
/// Panics if `x` and `y` have different lengths, as the mark constructors do.
pub fn m4(x: &[f64], y: &[f64], columns: usize) -> Option<(Vec<f64>, Vec<f64>)> {
    assert_eq!(x.len(), y.len(), "m4 requires series of equal length");
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

/// Reduces a line to at most four points per raster column, bucketing by the column
/// each point actually *renders* into (`map(x)` rounded to a subpixel column) rather
/// than by the raw x-domain. Because the buckets are the drawn pixel columns, the
/// reduction is pixel-exact for that raster — and it follows a non-linear axis (log)
/// for free, since `map` is the axis's own forward transform.
///
/// `x = None` means the implicit indices `0, 1, 2, …`, materialized on the fly.
/// Returns `None` when x is not ascending (M4 reorders within a column, exact only
/// for monotonic x); non-finite x, non-finite mapped positions (a non-positive value
/// on a log axis), and positions outside `[0, columns)` are skipped.
pub(crate) fn m4_mapped(
    x: Option<&[f64]>,
    y: &[f64],
    columns: usize,
    map: impl Fn(f64) -> f64,
) -> Option<(Vec<f64>, Vec<f64>)> {
    if columns == 0 {
        return None;
    }
    let mut aggregate = M4::columns(columns);
    let mut previous = f64::NEG_INFINITY;
    for (index, &yv) in y.iter().enumerate() {
        let xv = match x {
            Some(values) => values[index],
            None => index as f64,
        };
        if !xv.is_finite() {
            continue;
        }
        if xv < previous {
            return None;
        }
        previous = xv;
        let position = map(xv);
        if !position.is_finite() {
            continue;
        }
        let column = position.round();
        if (0.0..columns as f64).contains(&column) {
            aggregate.record(column as usize, xv, yv);
        }
    }
    Some(aggregate.emit())
}

#[cfg(test)]
#[path = "tests/m4_tests.rs"]
mod tests;
