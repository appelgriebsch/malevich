{
    use malevich::scale::Colormap;
    use malevich::{Cells, Line, Plot};
    let (w, h) = (36, 10);
    let field: Vec<f64> = (0..w * h).map(|i| {
        let (x, y) = ((i % w) as f64 / w as f64 * 4.0 - 2.0, (i / w) as f64 / h as f64 * 3.0 - 1.5);
        (-(x * x + y * y)).exp() * 8.0 + (x * 3.0).sin()
    }).collect();
    let x: Vec<f64> = (0..36).map(f64::from).collect();
    Plot::new()
        .layer(Cells::matrix(w, field).colormap(Colormap::VIRIDIS))
        .layer(Line::xy(x.clone(), x.iter().map(|x| 5.0 + 3.0 * (x * 0.3).sin()).collect::<Vec<_>>()).label("a line"))
        .colorbar()
        .title("a heatmap and a line")
}
