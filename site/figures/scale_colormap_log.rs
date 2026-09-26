{
    use malevich::scale::Colormap;
    use malevich::{Cells, Plot, Scale};
    // Attention weights span decades. A log colormap gives each decade an equal
    // share of the ramp, and the causal mask's zeros are gaps, not "very small".
    let tokens = ["The", "robot", "ate", "the", "red", "apple", "."];
    let n = tokens.len();
    let weights: Vec<f64> = (0..n * n).map(|i| {
        let (q, k) = (i / n, i % n);
        if k > q { f64::NAN } else { 10f64.powf(-((q - k) as f64) * 0.7) * (1.0 + 0.3 * ((q * 3 + k) as f64).sin()) }
    }).collect();
    Plot::new()
        .layer(Cells::matrix(n, weights).colormap(Colormap::MAGMA.log()))
        .x_scale(Scale::bands(tokens))
        .y_scale(Scale::bands(tokens))
        .colorbar()
        .title("MAGMA.log() over attention weights")
        .x_label("key")
        .y_label("query")
}
