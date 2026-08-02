//! A time axis: unix seconds in, calendar out. Ticks align to real boundaries and
//! labels show the unit that matters — midnight becomes the date, January becomes
//! the year. UTC, exact Gregorian arithmetic, no dependencies.

use malevich::{Frame, Line, Plot};

fn main() {
    // Three years of monthly readings with a trend and a seasonal swing.
    let month_stamp = |year: i64, month: u64| -> f64 {
        let y = year - i64::from(month <= 2);
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = (y - era * 400) as u64;
        let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        ((era * 146_097 + doe as i64 - 719_468) * 86_400) as f64
    };
    let stamps: Vec<f64> = (0..36)
        .map(|i| month_stamp(2024 + i / 12, (1 + i % 12) as u64))
        .collect();
    let level: Vec<f64> = (0..36)
        .map(|i| 400.0 + i as f64 * 0.2 + ((i % 12) as f64 * 0.52).sin() * 3.0)
        .collect();
    let chart = Plot::new()
        .layer(Line::xy(&stamps[..], &level[..]))
        .title("a monthly series on a calendar axis (synthetic)")
        .time_x();
    println!("{}", chart.render(&Frame::plain(64, 14)));
}
