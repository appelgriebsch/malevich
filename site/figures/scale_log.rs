{
    use malevich::{Plot, Points};
    // A power law renders straight on log–log axes, decades ticked on both.
    let x: Vec<f64> = (1..=60).map(|i| f64::from(i) * f64::from(i) * 0.7).collect();
    let y: Vec<f64> = x.iter().map(|x| 5e4 * x.powf(-1.6)).collect();
    Plot::new().layer(Points::xy(x, y)).log_x().log_y().title("a power law, rank against frequency")
}
