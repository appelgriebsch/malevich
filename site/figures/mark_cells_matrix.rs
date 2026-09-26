{
    use malevich::{Cells, Plot, Scale};
    // A value grid: row-major, rows labeled in matrix order on a band axis.
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];
    let hours = ["00", "04", "08", "12", "16", "20"];
    let mut values = Vec::new();
    for row in 0..6 {
        for column in 0..6 {
            values.push(20.0 + 18.0 * ((row as f64 - 2.5) * 0.7).cos() * ((column as f64 - 3.0) * 0.5).cos());
        }
    }
    Plot::new()
        .layer(Cells::matrix(6, values))
        .x_scale(Scale::bands(hours))
        .y_scale(Scale::bands(months))
        .colorbar()
        .title("Cells::matrix — a value grid under a colormap")
}
