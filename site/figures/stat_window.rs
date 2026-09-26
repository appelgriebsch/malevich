{
    use malevich::stat::{Reducer, Window};
    use malevich::{Line, Plot};
    let mut unit = super::noise(5);
    // Latency samples with a spiky tail; a rolling mean and a rolling p95 over 50.
    let samples: Vec<f64> = (0..400).map(|i| {
        let base = 40.0 + 10.0 * (f64::from(i) * 0.03).sin();
        if unit() < 0.06 { base + 60.0 * unit() } else { base + 6.0 * super::gaussian(&mut unit) }
    }).collect();
    let window = Window::new(50);
    Plot::new()
        .layer(Line::y(samples.clone()).label("samples"))
        .layer(Line::y(window.mean(&samples)).label("rolling mean"))
        .layer(Line::y(window.reduce(&samples, Reducer::Percentile(0.95))).label("rolling p95"))
        .title("Window: one reducer vocabulary")
        .y_label("ms")
}
