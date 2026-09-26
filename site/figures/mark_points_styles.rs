{
    use malevich::{Plot, PointStyle, Points};
    let styles = [
        (PointStyle::Dot, "Dot"), (PointStyle::Plus, "Plus"), (PointStyle::Cross, "Cross"),
        (PointStyle::Asterisk, "Asterisk"), (PointStyle::Circle, "Circle"),
    ];
    let mut plot = Plot::new();
    for (row, (style, name)) in styles.into_iter().enumerate() {
        let x: Vec<f64> = (0..8).map(|i| f64::from(i) * 1.5 + row as f64 * 0.3).collect();
        let y: Vec<f64> = x.iter().map(|x| row as f64 * 2.0 + (x * 0.8).sin() * 0.6).collect();
        plot = plot.layer(Points::xy(x, y).style(style).label(name));
    }
    plot.title("point styles")
}
