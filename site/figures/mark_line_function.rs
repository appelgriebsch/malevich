{
    use malevich::{Line, Plot};
    // A function samples itself once per subpixel column: no resolution to choose.
    Plot::new()
        .layer(Line::function(0.0..12.6, f64::sin).label("sin x"))
        .layer(Line::function(0.0..12.6, |x| (x * 0.5).cos() * 0.6).label("0.6 cos x/2"))
        .title("Line::function")
}
