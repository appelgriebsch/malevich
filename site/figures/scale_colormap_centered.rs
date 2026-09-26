{
    use malevich::scale::Colormap;
    use malevich::{Cells, Plot, Scale};
    // Signed data on a diverging map anchored at zero: the neutral color means zero,
    // whatever the range each side happens to span.
    let features = ["age", "len", "dep", "mass", "veg", "kcal", "spd", "alt"];
    let n = features.len();
    let grid: Vec<f64> = (0..n * n).map(|i| {
        let (row, column) = (i / n, i % n);
        if row == column { 1.0 } else { (-(row as f64 - column as f64).abs() * 0.35).exp() * ((row + column) as f64 * 0.55).cos() }
    }).collect();
    Plot::new()
        .layer(Cells::matrix(n, grid).colormap(Colormap::RED_BLUE.centered_at(0.0)))
        .x_scale(Scale::bands(features))
        .y_scale(Scale::bands(features))
        .colorbar()
        .title("RED_BLUE.centered_at(0.0)")
}
