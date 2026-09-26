{
    use malevich::{Line, Plot};
    let x: Vec<f64> = (0..200).map(f64::from).collect();
    let rate: Vec<f64> = x.iter().map(|v| 3.0 + (v * 0.05).sin()).collect();
    // Both panels fix the same x window; each keeps its own honest y.
    Plot::new().layer(Line::xy(x, rate)).x_domain(0.0, 199.0).title("rate").y_label("Hz")
}
