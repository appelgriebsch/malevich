{
    use malevich::{Line, Plot};
    use super::day_stamp;
    let mut unit = super::noise(37);
    // Six weeks of daily readings: day-of-month labels, the month and year once.
    let stamps: Vec<f64> = (0..42).map(|d| day_stamp(2026, 2, 10) + f64::from(d) * 86_400.0).collect();
    let temperature: Vec<f64> = stamps.iter().enumerate().map(|(i, _)| 4.0 + i as f64 * 0.25 + super::gaussian(&mut unit) * 1.5).collect();
    Plot::new().layer(Line::xy(stamps, temperature)).time_x().title("daily mean temperature").y_label("°C")
}
