{
    use malevich::stat::{stack_with, StackOffset, StackOptions};
    use malevich::{Area, Plot};
    let years: Vec<f64> = (2000..2026).map(f64::from).collect();
    let coal: Vec<f64> = years.iter().map(|y| 40.0 - (y - 2000.0) * 1.1).collect();
    let gas: Vec<f64> = years.iter().map(|y| 20.0 + (y - 2000.0) * 0.4).collect();
    let wind: Vec<f64> = years.iter().map(|y| 1.0 + ((y - 2000.0) * 0.35).powi(2)).collect();
    let solar: Vec<f64> = years.iter().map(|y| 0.2 + ((y - 2000.0) * 0.28).powi(2)).collect();
    // Normalize: every x sums to one, so the chart reads as shares.
    let options = StackOptions::new().offset(StackOffset::Normalize);
    let mut plot = Plot::new();
    for ((low, high), label) in stack_with(&[&coal, &gas, &wind, &solar], options).into_iter().zip(["coal", "gas", "wind", "solar"]) {
        plot = plot.layer(Area::between(years.clone(), low, high).label(label));
    }
    plot.title("StackOffset::Normalize — shares").x_label("year")
}
