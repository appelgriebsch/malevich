{
    use malevich::{Line, Plot};
    let mut unit = super::noise(21);
    // The same series, "reduced" the way a sampler would: every 400th point.
    // The spikes are gone, and nothing on the chart says so.
    let n = 200_000;
    let signal: Vec<f64> = (0..n).map(|i| {
        let t = i as f64 / n as f64;
        let wave = (t * 40.0).sin() * (t * 3.0).cos() * 2.0 + super::gaussian(&mut unit) * 0.2;
        if i == 31_337 || i == 99_999 || i == 150_001 { wave + 7.0 } else { wave }
    }).collect();
    let x: Vec<f64> = (0..n).step_by(400).map(|i| i as f64).collect();
    let sampled: Vec<f64> = signal.iter().step_by(400).cloned().collect();
    Plot::new().layer(Line::xy(x, sampled)).title("the same points, every 400th").y_min(-4.0).y_max(8.0)
}
