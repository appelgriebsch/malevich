{
    use malevich::scale::Colormap;
    use malevich::{Cells, Plot};
    // The ramp quantized into six bands the colorbar labels at their boundaries.
    let (w, h) = (48, 14);
    let field: Vec<f64> = (0..w * h).map(|i| {
        let (x, y) = ((i % w) as f64 / w as f64 * 4.0 - 2.0, (i / w) as f64 / h as f64 * 4.0 - 2.0);
        (-(x * x + y * y) * 0.6).exp() * 10.0 + (x * 3.0).sin()
    }).collect();
    Plot::new().layer(Cells::matrix(w, field).colormap(Colormap::CIVIDIS.steps(6))).colorbar().axes(false).title("Colormap::steps(6)")
}
