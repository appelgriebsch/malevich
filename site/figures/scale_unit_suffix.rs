{
    use malevich::scale::Unit;
    use malevich::{Line, Plot};
    let x: Vec<f64> = (0..60).map(f64::from).collect();
    let share: Vec<f64> = x.iter().map(|x| 50.0 + 45.0 * (x * 0.1).sin()).collect();
    Plot::new().layer(Line::xy(x, share)).y_unit(Unit::suffix("%")).title("Unit::suffix(\"%\")")
}
