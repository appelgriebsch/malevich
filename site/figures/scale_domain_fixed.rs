{
    use malevich::{Line, Plot};
    let x: Vec<f64> = (0..200).map(|i| f64::from(i) * 0.05).collect();
    let y: Vec<f64> = x.iter().map(|x| (x * 1.5).sin() * (x * 0.4).exp() * 0.2).collect();
    // Fixed both ways: what falls outside clips — it is drawn nowhere, never on the border.
    Plot::new().layer(Line::xy(x, y)).x_domain(2.0, 8.0).y_domain(-3.0, 3.0).title("x_domain and y_domain")
}
