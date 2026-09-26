{
    use malevich::scale::Unit;
    use malevich::{Line, Plot};
    let x: Vec<f64> = (0..60).map(f64::from).collect();
    let bytes: Vec<f64> = x.iter().map(|x| 2.0e6 * (1.0 + (x * 0.2).sin())).collect();
    Plot::new().layer(Line::xy(x, bytes)).y_unit(Unit::si("B")).title("Unit::si(\"B\")")
}
