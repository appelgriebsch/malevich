{
    use malevich::{Line, Plot};
    // The same furniture switch on a full plot: the data region fills the frame.
    let x: Vec<f64> = (0..200).map(|i| f64::from(i) * 0.05).collect();
    let y: Vec<f64> = x.iter().map(|x| (x * 1.3).sin() * (x * 0.2).cos()).collect();
    Plot::new().layer(Line::xy(x, y)).axes(false)
}
