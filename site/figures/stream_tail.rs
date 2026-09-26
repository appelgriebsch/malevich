{
    use malevich::{Line, Plot, Viewport};
    let mut unit = super::noise(61);
    let readings: Vec<f64> = (0..1000).map(|i| 20.0 + (f64::from(i) * 0.02).sin() * 5.0 + super::gaussian(&mut unit)).collect();
    // Follow the stream: a window ending at the latest index, 200 wide.
    Plot::new().layer(Line::y(readings)).viewport(Viewport::auto().tail(999.0, 200.0)).title("Viewport::tail — the last 200 of 1,000")
}
