{
    use malevich::{Line, Plot};
    let mut unit = super::noise(21);
    // 200,000 points with three one-sample spikes. Past four points per column
    // the plot reduces by M4 automatically: first, last, min, and max per
    // rendered column — the spikes cannot vanish, by construction.
    let n = 200_000;
    let signal: Vec<f64> = (0..n).map(|i| {
        let t = i as f64 / n as f64;
        let wave = (t * 40.0).sin() * (t * 3.0).cos() * 2.0 + super::gaussian(&mut unit) * 0.2;
        if i == 31_337 || i == 99_999 || i == 150_001 { wave + 7.0 } else { wave }
    }).collect();
    Plot::new().layer(Line::y(signal)).title("200,000 points through M4").y_min(-4.0).y_max(8.0)
}
