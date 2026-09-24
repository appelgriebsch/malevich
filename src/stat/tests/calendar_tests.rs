use super::{TimeUnit, calendar_bins};
use crate::scale::time::{DAY, HOUR, days_from_civil};
use crate::stat::Normalization;

fn stamp(year: i32, month: u32, day: u32) -> f64 {
    (days_from_civil(year, month, day) * DAY) as f64
}

#[test]
fn months_are_their_true_length_and_empties_stay() {
    let values = [
        stamp(2026, 1, 15),
        stamp(2026, 1, 20) + 3600.0,
        stamp(2026, 3, 3),
    ];
    let bins = calendar_bins(&values, TimeUnit::Month).unwrap();
    assert_eq!(
        bins.starts(),
        [stamp(2026, 1, 1), stamp(2026, 2, 1), stamp(2026, 3, 1)]
    );
    assert_eq!(
        bins.ends(),
        [stamp(2026, 2, 1), stamp(2026, 3, 1), stamp(2026, 4, 1)]
    );
    assert_eq!(bins.counts(), [2, 0, 1]);
    assert_eq!(bins.len(), 3);
    // A density divides by each month's own length: January is longer.
    let density = bins.heights(Normalization::Density, false);
    assert!((density[0] - 2.0 / (3.0 * 31.0 * DAY as f64)).abs() < 1e-15);
    assert!((density[2] - 1.0 / (3.0 * 31.0 * DAY as f64)).abs() < 1e-15);
    assert_eq!(
        bins.heights(Normalization::Percent, true),
        [200.0 / 3.0, 200.0 / 3.0, 100.0]
    );
}

#[test]
fn weeks_start_on_monday_and_years_in_january() {
    // 2026-08-01 is a Saturday; its week began on Monday the 27th of July.
    let saturday = stamp(2026, 8, 1) + 5.0 * HOUR as f64;
    let weeks = calendar_bins(&[saturday], TimeUnit::Week).unwrap();
    assert_eq!(weeks.starts(), [stamp(2026, 7, 27)]);
    assert_eq!(weeks.ends(), [stamp(2026, 8, 3)]);
    let years = calendar_bins(&[saturday, stamp(2027, 2, 1)], TimeUnit::Year).unwrap();
    assert_eq!(years.starts(), [stamp(2026, 1, 1), stamp(2027, 1, 1)]);
    assert_eq!(years.counts(), [1, 1]);
    let hours = calendar_bins(&[saturday, saturday + 59.0 * 60.0], TimeUnit::Hour).unwrap();
    assert_eq!(hours.len(), 1);
    assert_eq!(hours.counts(), [2]);
    let days = calendar_bins(&[saturday, saturday + 20.0 * HOUR as f64], TimeUnit::Day).unwrap();
    assert_eq!(days.counts(), [1, 1]);
}

#[test]
fn gaps_are_skipped_and_empty_input_is_none() {
    assert!(calendar_bins(&[], TimeUnit::Day).is_none());
    assert!(calendar_bins(&[f64::NAN], TimeUnit::Day).is_none());
    let bins = calendar_bins(&[f64::NAN, stamp(2026, 5, 5), f64::INFINITY], TimeUnit::Day).unwrap();
    assert_eq!(bins.counts(), [1]);
    // Beyond the supported calendar there are no buckets.
    assert!(calendar_bins(&[1e300], TimeUnit::Year).is_none());
}
