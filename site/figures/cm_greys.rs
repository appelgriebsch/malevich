{
    use malevich::scale::Colormap;
    use malevich::{Cells, Plot};
    let ramp: Vec<f64> = (0..64).map(f64::from).collect();
    Plot::new().layer(Cells::matrix(64, ramp).colormap(Colormap::GREYS)).axes(false).title("Colormap::GREYS")
}
