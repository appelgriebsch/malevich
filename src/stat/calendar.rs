//! Calendar bins: counts per hour, day, week, month, or year over unix
//! timestamps — buckets of their true length, empties included.

use crate::scale::time::{DAY, HOUR, civil_from_days, days_from_civil, supported_seconds};

/// A calendar unit, aligned as the time axis aligns it (UTC): hours and days
/// to their boundaries, weeks to Mondays, months and years to their firsts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum TimeUnit {
    /// Clock hours.
    Hour,
    /// Calendar days.
    Day,
    /// ISO weeks, Monday to Monday.
    Week,
    /// Calendar months, of their true length.
    Month,
    /// Calendar years.
    Year,
}

impl TimeUnit {
    /// The start of the bucket holding `t` (unix seconds).
    fn floor(self, t: i64) -> Option<i64> {
        let days = t.div_euclid(DAY);
        match self {
            TimeUnit::Hour => Some(t.div_euclid(HOUR) * HOUR),
            TimeUnit::Day => days.checked_mul(DAY),
            TimeUnit::Week => {
                // The epoch was a Thursday: three days on from a Monday.
                let weekday = (days + 3).rem_euclid(7);
                (days - weekday).checked_mul(DAY)
            }
            TimeUnit::Month => {
                let (year, month, _) = civil_from_days(days);
                days_from_civil(year, month, 1).checked_mul(DAY)
            }
            TimeUnit::Year => {
                let (year, ..) = civil_from_days(days);
                days_from_civil(year, 1, 1).checked_mul(DAY)
            }
        }
    }

    /// The start of the bucket after the one starting at `start`.
    fn next(self, start: i64) -> Option<i64> {
        match self {
            TimeUnit::Hour => start.checked_add(HOUR),
            TimeUnit::Day => start.checked_add(DAY),
            TimeUnit::Week => start.checked_add(7 * DAY),
            TimeUnit::Month => {
                let (year, month, _) = civil_from_days(start.div_euclid(DAY));
                let (year, month) = if month == 12 {
                    (year.checked_add(1)?, 1)
                } else {
                    (year, month + 1)
                };
                days_from_civil(year, month, 1).checked_mul(DAY)
            }
            TimeUnit::Year => {
                let (year, ..) = civil_from_days(start.div_euclid(DAY));
                days_from_civil(year.checked_add(1)?, 1, 1).checked_mul(DAY)
            }
        }
    }
}

/// Counts per calendar bucket over the finite extent of a series of unix
/// timestamps, every bucket present — the histogram whose bins are months
/// of their true length. Built by [`calendar_bins`]; the starts, ends, and
/// counts feed [`Bars::intervals`](crate::Bars::intervals) directly.
#[derive(Debug, Clone, PartialEq)]
pub struct CalendarBins {
    starts: Vec<f64>,
    ends: Vec<f64>,
    counts: Vec<u64>,
}

impl CalendarBins {
    /// The left edge of every bucket, in unix seconds.
    pub fn starts(&self) -> &[f64] {
        &self.starts
    }

    /// The right edge of every bucket — the next bucket's start.
    pub fn ends(&self) -> &[f64] {
        &self.ends
    }

    /// The per-bucket counts, in order.
    pub fn counts(&self) -> &[u64] {
        &self.counts
    }

    /// The number of buckets; at least one.
    pub fn len(&self) -> usize {
        self.counts.len()
    }

    /// Whether there are no buckets. Never true for a value [`calendar_bins`]
    /// returns.
    pub fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }

    /// The bar heights under `normalization`, accumulated when `cumulative`,
    /// as [`Bins::heights`](super::Bins::heights) defines them — a density
    /// divides each count by its own bucket's length in seconds, so months
    /// of different lengths compare honestly.
    pub fn heights(&self, normalization: super::Normalization, cumulative: bool) -> Vec<f64> {
        use super::Normalization;

        let total = self.counts.iter().sum::<u64>() as f64;
        let mut running = 0u64;
        self.counts
            .iter()
            .zip(self.starts.iter().zip(&self.ends))
            .map(|(&count, (&start, &end))| {
                running += count;
                let count = if cumulative { running } else { count } as f64;
                match normalization {
                    Normalization::Count => count,
                    _ if total == 0.0 => 0.0,
                    Normalization::Probability => count / total,
                    Normalization::Percent => 100.0 * count / total,
                    Normalization::Density if cumulative => count / total,
                    Normalization::Density => count / (total * (end - start)),
                }
            })
            .collect()
    }
}

/// Counts `values` (unix seconds, UTC) per calendar `unit` over their finite
/// extent, empties included. `None` when no value is finite, when the extent
/// leaves the supported calendar (years −999999 through 999999), or when it
/// would take more buckets than the statistics budget allows.
pub fn calendar_bins(values: &[f64], unit: TimeUnit) -> Option<CalendarBins> {
    let mut extent: Option<(f64, f64)> = None;
    for &value in values {
        if value.is_finite() {
            let (lo, hi) = extent.get_or_insert((value, value));
            *lo = lo.min(value);
            *hi = hi.max(value);
        }
    }
    let (lo, hi) = extent?;
    let (supported_lo, supported_hi) = supported_seconds();
    if lo < supported_lo as f64 || hi > supported_hi as f64 {
        return None;
    }
    let (lo, hi) = (lo.floor() as i64, hi.floor() as i64);
    let mut starts = vec![unit.floor(lo)?];
    let mut ends = Vec::new();
    loop {
        let next = unit.next(*starts.last()?)?;
        ends.push(next);
        if next > hi {
            break;
        }
        if starts.len() >= super::MAX_STAT_ELEMENTS {
            return None;
        }
        starts.push(next);
    }
    let mut counts = vec![0u64; starts.len()];
    for &value in values {
        if value.is_finite() {
            let t = value.floor() as i64;
            let bucket = starts
                .partition_point(|&start| start <= t)
                .saturating_sub(1);
            counts[bucket] += 1;
        }
    }
    Some(CalendarBins {
        starts: starts.into_iter().map(|t| t as f64).collect(),
        ends: ends.into_iter().map(|t| t as f64).collect(),
        counts,
    })
}

#[cfg(test)]
#[path = "tests/calendar_tests.rs"]
mod tests;
