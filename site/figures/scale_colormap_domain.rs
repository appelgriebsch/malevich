{
    use malevich::scale::Colormap;
    use malevich::{Cells, Color, Plot};
    // A fixed color domain so two grids read on one scale; `under` and `over`
    // disclose what falls outside instead of clamping it into the ramp's ends.
    let (w, h) = (48, 12);
    let field: Vec<f64> = (0..w * h).map(|i| {
        let (x, y) = ((i % w) as f64 / w as f64, (i / w) as f64 / h as f64);
        60.0 * (x * 6.0).sin() * (y * 4.0).cos()
    }).collect();
    let colormap = Colormap::VIRIDIS.domain(-30.0, 30.0).under(Color::Rgb(70, 20, 90)).over(Color::Rgb(255, 60, 40));
    Plot::new().layer(Cells::matrix(w, field).colormap(colormap)).colorbar().axes(false).title("domain(-30, 30) with under and over")
}
