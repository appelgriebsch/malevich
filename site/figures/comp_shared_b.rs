{
    use malevich::{Line, Plot};
    let x: Vec<f64> = (0..200).map(f64::from).collect();
    let count: Vec<f64> = x.iter().map(|v| 1000.0 + v * 12.0 + (v * 0.3).sin() * 80.0).collect();
    Plot::new().layer(Line::xy(x, count)).x_domain(0.0, 199.0).title("count")
}
