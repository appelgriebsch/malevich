{
    use malevich::{Dash, Line, LineStyle, Plot};
    let x: Vec<f64> = (0..40).map(|i| f64::from(i) * 0.25).collect();
    let wave: Vec<f64> = x.iter().map(|x| (x * 1.1).sin() * 4.0 + 5.0).collect();
    let lower: Vec<f64> = wave.iter().map(|y| y - 4.5).collect();
    Plot::new()
        .layer(Line::xy(x.clone(), wave).label("Pixels, the default"))
        .layer(Line::xy(x.clone(), lower.clone()).style(LineStyle::Corners).label("Corners, one glyph per column"))
        .layer(Line::xy(x, lower.iter().map(|y| y - 4.5).collect::<Vec<_>>()).dash(Dash::Dashed).label("dashed"))
        .title("line styles")
}
