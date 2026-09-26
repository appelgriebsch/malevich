{
    use malevich::stat::{calendar_bins, TimeUnit};
    use malevich::{Bars, Plot};
    use super::day_stamp;
    let mut unit = super::noise(29);
    // Incidents over a year: counted per calendar month, each bar drawn between
    // its month's true edges on the time axis (February is shorter).
    let stamps: Vec<f64> = (0..300).map(|_| day_stamp(2025, 1, 1) + unit() * 365.0 * 86_400.0 * (0.4 + 0.6 * unit())).collect();
    let months = calendar_bins(&stamps, TimeUnit::Month).expect("data");
    let counts: Vec<f64> = months.counts().iter().map(|&c| c as f64).collect();
    Plot::new()
        .layer(Bars::intervals(months.starts().to_vec(), months.ends().to_vec(), counts))
        .time_x()
        .title("stat::calendar_bins — incidents per month")
}
