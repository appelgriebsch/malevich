{
    use malevich::{Line, Plot};
    let x: Vec<f64> = (0..200).map(|i| f64::from(i) * 0.05).collect();
    let y: Vec<f64> = x.iter().map(|x| (x * 1.5).sin() * (x * 0.4).exp() * 0.2).collect();
    Plot::new().layer(Line::xy(x, y)).title("the automatic domain")
}
