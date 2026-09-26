{
    use malevich::{Area, Plot};
    let mut unit = super::noise(41);
    let traffic: Vec<f64> = (0..120).map(|i| 300.0 + 240.0 * (f64::from(i) * 0.08).sin().max(0.0) + unit() * 40.0).collect();
    // A rate chart floored at zero: the top follows the traffic and grows to its outer tick.
    Plot::new().layer(Area::y(traffic)).y_min(0.0).title("y_min(0.0)").y_label("req/s")
}
