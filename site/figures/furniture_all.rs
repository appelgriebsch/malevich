{
    use malevich::scale::Colormap;
    use malevich::{Cells, Line, Plot};
    // Every piece of furniture at once: title, axis labels, a legend, a colorbar.
    let (w, h) = (40, 12);
    let field: Vec<f64> = (0..w * h).map(|i| ((i % w) as f64 * 0.3).sin() * ((i / w) as f64 * 0.5).cos()).collect();
    let x: Vec<f64> = (0..40).map(f64::from).collect();
    let ridge: Vec<f64> = x.iter().map(|x| 6.0 + 3.0 * (x * 0.2).sin()).collect();
    Plot::new()
        .layer(Cells::matrix(w, field).colormap(Colormap::CIVIDIS))
        .layer(Line::xy(x, ridge).label("ridge").glow())
        .colorbar()
        .title("the furniture")
        .x_label("column")
        .y_label("row")
}
