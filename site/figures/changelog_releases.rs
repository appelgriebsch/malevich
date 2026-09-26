{
    use malevich::stat::{calendar_bins, TimeUnit};
    use malevich::{Bars, Plot, Scale};
    use super::release_dates;
    // Releases per ISO week, counted from the changelog's own headings.
    let dates = release_dates();
    let months = calendar_bins(&dates, TimeUnit::Week).expect("releases");
    let counts: Vec<f64> = months.counts().iter().map(|&c| c as f64).collect();
    Plot::new()
        .layer(Bars::intervals(months.starts().to_vec(), months.ends().to_vec(), counts))
        .time_x()
        .y_scale(Scale::Integer)
        .title(format!("{} releases, by week", dates.len()))
}
