{
    use malevich::stat::{cumsum, normalize, Reducer};
    use malevich::{Line, Plot};
    let mut unit = super::noise(23);
    let daily: Vec<f64> = (0..90).map(|i| 3.0 + (f64::from(i) * 0.15).sin() * 2.0 + unit()).collect();
    // The series maps: a running total, and a series divided by a reducer of itself.
    Plot::new()
        .layer(Line::y(normalize(&daily, Reducer::Max)).label("daily, as a share of the peak"))
        .layer(Line::y(normalize(&cumsum(&daily), Reducer::Max)).label("cumulative, share of the total"))
        .title("cumsum and normalize")
        .x_label("day")
}
