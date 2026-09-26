{
    use malevich::{Line, Plot};
    let mut unit = super::noise(51);
    let n = 100_000;
    let signal: Vec<f64> = (0..n).map(|i| {
        let t = i as f64 / n as f64;
        (t * 60.0).sin() * 3.0 + (t * 700.0).sin() * 0.4 + super::gaussian(&mut unit) * 0.1 + if i == 42_000 { 5.0 } else { 0.0 }
    }).collect();
    Plot::new().layer(Line::y(signal)).title("100,000 points, the whole window")
}
