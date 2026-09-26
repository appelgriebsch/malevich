{
    use malevich::scale::Colormap;
    use malevich::stat::Reducer;
    use malevich::{Cells, Plot};
    // A 600×300 grid onto a few dozen cells: every screen bucket owns the cells
    // whose centers fall inside it and shows their maximum, so the sparse
    // spikes survive. Extents place the grid in data coordinates.
    let (w, h) = (600, 300);
    let mut unit = super::noise(11);
    let mut field = Vec::with_capacity(w * h);
    for row in 0..h {
        for column in 0..w {
            let (x, y) = (column as f64 / 100.0, row as f64 / 100.0);
            let base = ((x * 2.0).sin() * (y * 3.0).cos()).abs() * 0.3;
            field.push(if unit() < 0.0008 { 1.0 } else { base });
        }
    }
    Plot::new()
        .layer(Cells::matrix(w, field).extents((0.0, 6.0), (0.0, 3.0)).reduce(Reducer::Max).colormap(Colormap::CIVIDIS))
        .colorbar()
        .title("180,000 cells, max-reduced")
        .x_label("s")
        .y_label("kHz")
}
