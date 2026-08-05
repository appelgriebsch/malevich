//! The `--time-x` input parser: unix seconds, or a hand-rolled ISO subset (D-C7).
//!
//! Accepted: a numeric epoch (seconds, or milliseconds when the magnitude is past
//! ~1e11), and `YYYY-MM-DD[ T HH:MM[:SS[.fff]]][Z|±HH:MM]`. No chrono, no jiff —
//! the differentiating part is small enough to own, and everything outside the
//! subset is a job for `date +%s` upstream. The core axis is unix seconds (f64,
//! UTC); a value that will not parse becomes an honest gap.

/// Parses one field to unix seconds, or `None` if it is neither an epoch nor an
/// ISO timestamp in the accepted subset.
pub fn parse(field: &str) -> Option<f64> {
    let text = field.trim();
    if text.is_empty() {
        return None;
    }
    // A bare number is an epoch: seconds, or milliseconds once it is implausibly
    // large for seconds (~1e11 s is year 5138; past it, read as milliseconds).
    if let Ok(number) = text.parse::<f64>() {
        if !number.is_finite() {
            return None;
        }
        return Some(if number.abs() >= 1e11 {
            number / 1000.0
        } else {
            number
        });
    }
    parse_iso(text)
}

/// Parses the ISO subset into unix seconds.
fn parse_iso(text: &str) -> Option<f64> {
    // Date and (optional) time are split by `T` or a space.
    let (date, rest) = match text.split_once(['T', ' ']) {
        Some((date, rest)) => (date, Some(rest)),
        None => (text, None),
    };
    let mut parts = date.split('-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = two(parts.next()?)?;
    let day: i64 = two(parts.next()?)?;
    if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let mut seconds = days_from_civil(year, month, day) as f64 * 86_400.0;

    if let Some(rest) = rest {
        // Peel a trailing timezone (`Z` or `±HH:MM`) off the clock.
        let (clock, offset) = split_timezone(rest)?;
        seconds += clock_seconds(clock)?;
        seconds -= offset;
    }
    Some(seconds)
}

/// Splits a `HH:MM[:SS[.fff]]` clock from an optional trailing timezone, returning
/// the clock text and the timezone's offset in seconds (0 without one, or with `Z`).
fn split_timezone(rest: &str) -> Option<(&str, f64)> {
    if let Some(clock) = rest.strip_suffix('Z') {
        return Some((clock, 0.0));
    }
    // The clock (`HH:MM:SS.fff`) never carries a sign, so the first `+`/`-` in the
    // remainder introduces the timezone — even though the offset has its own colon.
    if let Some(sign) = rest.find(['+', '-']) {
        let (clock, zone) = rest.split_at(sign);
        let (zh, zm) = zone[1..].split_once(':')?;
        let offset = (two(zh)? * 3600 + two(zm)? * 60) as f64;
        let offset = if zone.starts_with('-') {
            -offset
        } else {
            offset
        };
        return Some((clock, offset));
    }
    Some((rest, 0.0))
}

/// Seconds since midnight for a `HH:MM[:SS[.fff]]` clock.
fn clock_seconds(clock: &str) -> Option<f64> {
    let mut parts = clock.split(':');
    let hour = two(parts.next()?)?;
    let minute = two(parts.next()?)?;
    if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) {
        return None;
    }
    let mut seconds = (hour * 3600 + minute * 60) as f64;
    if let Some(sec) = parts.next() {
        // Seconds may carry a fraction; parse the whole `SS.fff` as f64.
        let value: f64 = sec.parse().ok()?;
        if !(0.0..60.0).contains(&value) {
            return None;
        }
        seconds += value;
    }
    if parts.next().is_some() {
        return None;
    }
    Some(seconds)
}

/// Parses a two-or-fewer-digit field, rejecting signs and junk.
fn two(text: &str) -> Option<i64> {
    if text.is_empty() || text.len() > 2 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// Days from the unix epoch (1970-01-01) to `y-m-d`, by Howard Hinnant's
/// civil-from-days algorithm. Valid for the proleptic Gregorian calendar.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
#[path = "tests/time_tests.rs"]
mod tests;
