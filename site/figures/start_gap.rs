{
    use malevich::{Line, Plot};
    // A reading the sensor missed is NaN: a gap, never a line drawn across it.
    let readings = vec![3.0, 3.4, 3.1, f64::NAN, f64::NAN, 2.2, 2.6, 2.9, 3.3];
    Plot::new().layer(Line::y(readings)).title("a gap is a gap")
}
