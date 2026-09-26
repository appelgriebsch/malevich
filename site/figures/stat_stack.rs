{
    use malevich::stat::stack;
    use malevich::{Area, Plot};
    let years: Vec<f64> = (2000..2026).map(f64::from).collect();
    let coal: Vec<f64> = years.iter().map(|y| 40.0 - (y - 2000.0) * 1.1).collect();
    let gas: Vec<f64> = years.iter().map(|y| 20.0 + (y - 2000.0) * 0.4).collect();
    let wind: Vec<f64> = years.iter().map(|y| 1.0 + ((y - 2000.0) * 0.35).powi(2)).collect();
    let solar: Vec<f64> = years.iter().map(|y| 0.2 + ((y - 2000.0) * 0.28).powi(2)).collect();
    // stack returns (low, high) per series; each Area sits on the sum of the ones below.
    let mut plot = Plot::new();
    for ((low, high), label) in stack(&[&coal, &gas, &wind, &solar]).into_iter().zip(["coal", "gas", "wind", "solar"]) {
        plot = plot.layer(Area::between(years.clone(), low, high).label(label));
    }
    plot.title("stat::stack — a stacked area").x_label("year").y_label("TWh")
}
