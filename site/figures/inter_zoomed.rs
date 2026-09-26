{
    use malevich::{Line, Plot, Viewport};
    let mut unit = super::noise(51);
    let n = 100_000;
    let signal: Vec<f64> = (0..n).map(|i| {
        let t = i as f64 / n as f64;
        (t * 60.0).sin() * 3.0 + (t * 700.0).sin() * 0.4 + super::gaussian(&mut unit) * 0.1 + if i == 42_000 { 5.0 } else { 0.0 }
    }).collect();
    // A zoom is a domain window. M4 re-aggregates to the columns of the new window,
    // so the ripple the wide view could only hint at is drawn in full.
    Plot::new().layer(Line::y(signal)).viewport(Viewport::auto().with_x(41_000.0, 44_000.0)).title("the same plot, zoomed")
}
