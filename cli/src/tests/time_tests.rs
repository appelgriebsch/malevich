use super::*;

#[test]
fn the_epoch_is_zero() {
    assert_eq!(parse("1970-01-01T00:00:00Z"), Some(0.0));
    assert_eq!(parse("1970-01-01"), Some(0.0));
}

#[test]
fn known_dates_match_their_unix_seconds() {
    assert_eq!(parse("2000-01-01T00:00:00Z"), Some(946_684_800.0));
    assert_eq!(parse("2021-01-01"), Some(1_609_459_200.0));
    assert_eq!(parse("2021-01-01T12:00:00Z"), Some(1_609_502_400.0));
}

#[test]
fn a_space_separates_date_and_time_like_sql() {
    assert_eq!(parse("2021-01-01 12:00:00"), Some(1_609_502_400.0));
}

#[test]
fn minutes_without_seconds_are_accepted() {
    assert_eq!(parse("2021-01-01T00:01"), Some(1_609_459_260.0));
}

#[test]
fn fractional_seconds_survive() {
    assert_eq!(parse("2021-01-01T00:00:00.5Z"), Some(1_609_459_200.5));
}

#[test]
fn a_positive_offset_shifts_back_to_utc() {
    // 00:00 at +01:00 is 23:00 the previous day in UTC.
    assert_eq!(parse("2021-01-01T00:00:00+01:00"), Some(1_609_455_600.0));
}

#[test]
fn a_negative_offset_shifts_forward() {
    assert_eq!(parse("2021-01-01T00:00:00-05:00"), Some(1_609_477_200.0));
}

#[test]
fn numeric_epoch_seconds_pass_through() {
    assert_eq!(parse("1609459200"), Some(1_609_459_200.0));
    assert_eq!(parse("0"), Some(0.0));
    assert_eq!(parse("-100"), Some(-100.0));
}

#[test]
fn large_numbers_are_read_as_milliseconds() {
    assert_eq!(parse("1609459200000"), Some(1_609_459_200.0));
}

#[test]
fn junk_and_out_of_range_are_gaps() {
    assert_eq!(parse(""), None);
    assert_eq!(parse("not-a-date"), None);
    assert_eq!(parse("2021-13-01"), None); // month 13
    assert_eq!(parse("2021-01-32"), None); // day 32
    assert_eq!(parse("2021-01-01T25:00"), None); // hour 25
    assert_eq!(parse("2021-01-01T00:61"), None); // minute 61
    assert_eq!(parse("2021-01"), None); // no day
    assert_eq!(parse("inf"), None);
}
