{
    use malevich::{Line, Plot};
    // Values agreeing in their leading digits read relative to a base the axis
    // prints once — the context note — instead of repeating it on every label.
    let x: Vec<f64> = (0..80).map(f64::from).collect();
    let y: Vec<f64> = x.iter().map(|x| 1.000_012e9 + 40.0 * (x * 0.2).sin()).collect();
    Plot::new().layer(Line::xy(x, y)).title("a context note for the digits the labels share").y_label("samples")
}
