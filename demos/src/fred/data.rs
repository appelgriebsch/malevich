//! The data layer: FRED CSV parsing, calendar arithmetic, and pure transforms.
//!
//! Nothing here touches a terminal or draws — everything is data in, data out,
//! which is what makes it unit-testable and lets the views stay declarative.

use std::collections::HashMap;

/// How a series' values behave under change arithmetic.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A level or index (GDP, CPI, payrolls): changes are percent.
    Index,
    /// Already a rate in percent (unemployment, yields): changes are points.
    Rate,
}

/// One economic series: identity, sampling, and observations.
pub struct Series {
    pub id: &'static str,
    pub title: &'static str,
    pub source: &'static str,
    pub unit: &'static str,
    /// Observations per year — 12 monthly, 4 quarterly.
    pub per_year: usize,
    pub kind: Kind,
    /// Unix seconds (UTC midnight of the observation date), ascending.
    pub dates: Vec<f64>,
    /// Observed values; missing observations are `NaN` gaps.
    pub values: Vec<f64>,
}

impl Series {
    /// The most recent finite observation.
    pub fn latest(&self) -> Option<f64> {
        self.values.iter().rev().copied().find(|v| v.is_finite())
    }

    /// The finite `(min, max)` of the values.
    pub fn extent(&self) -> (f64, f64) {
        extent(&self.values)
    }

    /// The change over one year at each date: percent for an index, points for a
    /// rate. The first year is `NaN` — there is nothing to compare against.
    pub fn year_over_year(&self) -> Vec<f64> {
        let step = self.per_year;
        let mut out = vec![f64::NAN; self.values.len()];
        let pairs = self
            .values
            .iter()
            .zip(&self.values[step.min(self.values.len())..]);
        for (slot, (&then, &now)) in out[step.min(self.values.len())..].iter_mut().zip(pairs) {
            if now.is_finite() && then.is_finite() {
                *slot = match self.kind {
                    Kind::Index if then != 0.0 => (now / then - 1.0) * 100.0,
                    Kind::Index => f64::NAN,
                    Kind::Rate => now - then,
                };
            }
        }
        out
    }

    /// The most recent year-over-year change, if any.
    pub fn latest_year_change(&self) -> Option<f64> {
        self.year_over_year()
            .iter()
            .rev()
            .copied()
            .find(|v| v.is_finite())
    }

    /// Period-over-period changes (percent for an index, points for a rate),
    /// aligned to the later observation; finite entries only.
    pub fn period_changes(&self) -> Vec<f64> {
        self.values
            .windows(2)
            .filter_map(|pair| {
                let (then, now) = (pair[0], pair[1]);
                if !(then.is_finite() && now.is_finite()) {
                    return None;
                }
                match self.kind {
                    Kind::Index if then != 0.0 => Some((now / then - 1.0) * 100.0),
                    Kind::Index => None,
                    Kind::Rate => Some(now - then),
                }
            })
            .collect()
    }

    /// Finite values grouped by decade, oldest first: `("1970s", values…)`.
    /// Decades with fewer than eight observations are dropped.
    pub fn by_decade(&self) -> (Vec<String>, Vec<Vec<f64>>) {
        let mut groups: Vec<(i64, Vec<f64>)> = Vec::new();
        for (&date, &value) in self.dates.iter().zip(&self.values) {
            if !value.is_finite() {
                continue;
            }
            let decade = year_of(date) / 10 * 10;
            match groups.last_mut() {
                Some((current, values)) if *current == decade => values.push(value),
                _ => groups.push((decade, vec![value])),
            }
        }
        groups.retain(|(_, values)| values.len() >= 8);
        let labels = groups.iter().map(|(d, _)| format!("{d}s")).collect();
        let values = groups.into_iter().map(|(_, v)| v).collect();
        (labels, values)
    }

    /// A period-change grid for the seasonality heatmap: one row per year (oldest at
    /// the bottom, matching Cells row order), one column per period, spanning the
    /// most recent `max_rows` years. Returns `(columns, values, first_year, last_year)`.
    pub fn seasonality(&self, max_rows: usize) -> (usize, Vec<f64>, i64, i64) {
        let columns = self.per_year;
        let changes: Vec<(i64, usize, f64)> = self
            .dates
            .windows(2)
            .zip(self.values.windows(2))
            .filter_map(|(dates, pair)| {
                let (then, now) = (pair[0], pair[1]);
                if !(then.is_finite() && now.is_finite()) {
                    return None;
                }
                let change = match self.kind {
                    Kind::Index if then != 0.0 => (now / then - 1.0) * 100.0,
                    Kind::Index => return None,
                    Kind::Rate => now - then,
                };
                let (year, month) = year_month(dates[1]);
                let column = (month as usize - 1) * columns / 12;
                Some((year, column, change))
            })
            .collect();
        let Some(&(last_year, ..)) = changes.last() else {
            return (columns, Vec::new(), 0, 0);
        };
        let first_year = last_year - max_rows as i64 + 1;
        let rows = max_rows;
        let mut grid = vec![f64::NAN; rows * columns];
        for (year, column, change) in changes {
            if year >= first_year {
                let row = (year - first_year) as usize;
                grid[row * columns + column] = change;
            }
        }
        (columns, grid, first_year, last_year)
    }
}

/// The full data set: every vendored series plus the NBER recession calendar.
pub struct Catalog {
    pub series: Vec<Series>,
    /// Recession periods as `(start, end)` in unix seconds.
    pub recessions: Vec<(f64, f64)>,
}

impl Catalog {
    /// Loads the vendored snapshot (see `demos/data/README.md`).
    pub fn load() -> Catalog {
        let spec: [(&str, &str, &str, &str, usize, Kind, &str); 6] = [
            (
                "UNRATE",
                "Unemployment Rate",
                "U.S. Bureau of Labor Statistics",
                "%",
                12,
                Kind::Rate,
                include_str!("../../data/unrate.csv"),
            ),
            (
                "CPIAUCSL",
                "Consumer Price Index (all items)",
                "U.S. Bureau of Labor Statistics",
                "index",
                12,
                Kind::Index,
                include_str!("../../data/cpi.csv"),
            ),
            (
                "GDPC1",
                "Real Gross Domestic Product",
                "U.S. Bureau of Economic Analysis",
                "bil. chained $",
                4,
                Kind::Index,
                include_str!("../../data/gdp.csv"),
            ),
            (
                "FEDFUNDS",
                "Federal Funds Effective Rate",
                "Federal Reserve Board",
                "%",
                12,
                Kind::Rate,
                include_str!("../../data/fedfunds.csv"),
            ),
            (
                "GS10",
                "10-Year Treasury Yield",
                "Federal Reserve Board",
                "%",
                12,
                Kind::Rate,
                include_str!("../../data/gs10.csv"),
            ),
            (
                "PAYEMS",
                "Total Nonfarm Payrolls",
                "U.S. Bureau of Labor Statistics",
                "thousands",
                12,
                Kind::Index,
                include_str!("../../data/payems.csv"),
            ),
        ];
        Catalog {
            series: spec
                .into_iter()
                .map(|(id, title, source, unit, per_year, kind, csv)| {
                    let (dates, values) = parse_csv(csv);
                    Series {
                        id,
                        title,
                        source,
                        unit,
                        per_year,
                        kind,
                        dates,
                        values,
                    }
                })
                .collect(),
            recessions: recession_periods(include_str!("../../data/recession.csv")),
        }
    }

    /// The series with this FRED id.
    pub fn by_id(&self, id: &str) -> Option<&Series> {
        self.series.iter().find(|s| s.id.eq_ignore_ascii_case(id))
    }
}

/// Parses a FRED CSV (`observation_date,VALUE`) into unix-second dates and values;
/// `.` (missing) becomes a `NaN` gap.
pub fn parse_csv(csv: &str) -> (Vec<f64>, Vec<f64>) {
    let mut dates = Vec::new();
    let mut values = Vec::new();
    for line in csv.lines().skip(1) {
        let Some((date, value)) = line.split_once(',') else {
            continue;
        };
        let Some(seconds) = parse_date(date) else {
            continue;
        };
        dates.push(seconds);
        values.push(value.trim().parse().unwrap_or(f64::NAN));
    }
    (dates, values)
}

/// Contiguous recession periods `(start, end)` from the NBER 0/1 indicator series.
pub fn recession_periods(csv: &str) -> Vec<(f64, f64)> {
    let (dates, flags) = parse_csv(csv);
    let mut periods = Vec::new();
    let mut start: Option<f64> = None;
    for (&date, &flag) in dates.iter().zip(&flags) {
        match (flag >= 0.5, start) {
            (true, None) => start = Some(date),
            (false, Some(begin)) => {
                periods.push((begin, date));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(begin) = start
        && let Some(&last) = dates.last()
    {
        periods.push((begin, last));
    }
    periods
}

/// Doubles interior points so a series draws as steps: the value holds flat until
/// the next observation. Honest for administered rates like the fed funds rate.
pub fn step_series(x: &[f64], y: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let mut sx = Vec::with_capacity(x.len() * 2);
    let mut sy = Vec::with_capacity(y.len() * 2);
    for index in 0..x.len().min(y.len()) {
        if index > 0 {
            sx.push(x[index]);
            sy.push(sy.last().copied().unwrap_or(y[index]));
        }
        sx.push(x[index]);
        sy.push(y[index]);
    }
    (sx, sy)
}

/// Pairs two series on their shared dates, keeping only rows where both values are
/// finite: `(dates, a_values, b_values)`.
pub fn align(
    a_dates: &[f64],
    a_values: &[f64],
    b_dates: &[f64],
    b_values: &[f64],
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let b_by_date: HashMap<i64, f64> = b_dates
        .iter()
        .zip(b_values)
        .map(|(&d, &v)| (d as i64, v))
        .collect();
    let mut dates = Vec::new();
    let mut left = Vec::new();
    let mut right = Vec::new();
    for (&date, &value) in a_dates.iter().zip(a_values) {
        if let Some(&other) = b_by_date.get(&(date as i64))
            && value.is_finite()
            && other.is_finite()
        {
            dates.push(date);
            left.push(value);
            right.push(other);
        }
    }
    (dates, left, right)
}

/// The finite `(min, max)` of a slice.
pub fn extent(values: &[f64]) -> (f64, f64) {
    values
        .iter()
        .filter(|v| v.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        })
}

/// `YYYY-MM-DD` to unix seconds (UTC midnight).
pub fn parse_date(date: &str) -> Option<f64> {
    let mut parts = date.split('-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = parts.next()?.parse().ok()?;
    Some(days_from_civil(year, month, day) as f64 * 86_400.0)
}

/// Days since the unix epoch for a proleptic-Gregorian date — Howard Hinnant's
/// algorithm, the same civil arithmetic malevich's time axis uses.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// `(year, month)` of a unix timestamp — the inverse civil conversion.
pub fn year_month(seconds: f64) -> (i64, u32) {
    let days = (seconds / 86_400.0).floor() as i64 + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    (year + i64::from(month <= 2), month as u32)
}

/// The calendar year of a unix timestamp.
pub fn year_of(seconds: f64) -> i64 {
    year_month(seconds).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_round_trip_through_the_civil_conversions() {
        for date in ["1948-01-01", "1970-01-01", "2000-02-29", "2026-08-01"] {
            let seconds = parse_date(date).unwrap();
            let (year, month) = year_month(seconds);
            let expected: Vec<i64> = date.split('-').map(|p| p.parse().unwrap()).collect();
            assert_eq!((year, month as i64), (expected[0], expected[1]), "{date}");
        }
        assert_eq!(parse_date("1970-01-01"), Some(0.0));
    }

    fn monthly(values: Vec<f64>, kind: Kind) -> Series {
        let dates = (0..values.len()).map(|i| i as f64 * 2_629_800.0).collect();
        Series {
            id: "TEST",
            title: "test",
            source: "test",
            unit: "",
            per_year: 12,
            kind,
            dates,
            values,
        }
    }

    #[test]
    fn year_over_year_is_percent_for_indexes_and_points_for_rates() {
        let index = monthly(
            (0..24).map(|i| 100.0 * 1.01f64.powi(i)).collect(),
            Kind::Index,
        );
        let yoy = index.year_over_year();
        assert!(yoy[..12].iter().all(|v| v.is_nan()));
        assert!((yoy[12] - (1.01f64.powi(12) - 1.0) * 100.0).abs() < 1e-9);

        let rate = monthly((0..24).map(|i| 3.0 + i as f64 * 0.1).collect(), Kind::Rate);
        let yoy = rate.year_over_year();
        assert!((yoy[12] - 1.2).abs() < 1e-9, "twelve steps of 0.1 points");
    }

    #[test]
    fn steps_hold_the_previous_value_until_the_next_observation() {
        let (sx, sy) = step_series(&[0.0, 1.0, 2.0], &[5.0, 7.0, 6.0]);
        assert_eq!(sx, [0.0, 1.0, 1.0, 2.0, 2.0]);
        assert_eq!(sy, [5.0, 5.0, 7.0, 7.0, 6.0]);
    }

    #[test]
    fn recession_periods_pair_up_and_close_at_the_end() {
        let csv = "observation_date,USREC\n1970-01-01,0\n1970-02-01,1\n1970-03-01,1\n1970-04-01,0\n1970-05-01,1\n";
        let periods = recession_periods(csv);
        assert_eq!(periods.len(), 2);
        assert!(periods[0].0 < periods[0].1);
        assert_eq!(
            periods[1].0, periods[1].1,
            "an open recession closes at the last date"
        );
    }

    #[test]
    fn alignment_keeps_only_shared_finite_dates() {
        let (dates, a, b) = align(
            &[0.0, 100.0, 200.0],
            &[1.0, 2.0, 3.0],
            &[100.0, 200.0, 300.0],
            &[20.0, f64::NAN, 30.0],
        );
        assert_eq!(dates, [100.0]);
        assert_eq!(a, [2.0]);
        assert_eq!(b, [20.0]);
    }
}
