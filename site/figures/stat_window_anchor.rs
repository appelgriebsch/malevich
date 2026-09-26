{
    use malevich::stat::{Window, WindowAnchor};
    use malevich::{Line, Plot};
    let mut unit = super::noise(8);
    let signal: Vec<f64> = (0..200).map(|i| if (60..70).contains(&i) { 10.0 } else { 2.0 + super::gaussian(&mut unit) * 0.4 }).collect();
    // A trailing window lags the step; a centered one straddles it; `strict`
    // gaps the positions whose window is incomplete instead of guessing.
    Plot::new()
        .layer(Line::y(signal.clone()).label("signal"))
        .layer(Line::y(Window::new(21).mean(&signal)).label("trailing"))
        .layer(Line::y(Window::new(21).anchor(WindowAnchor::Middle).strict().mean(&signal)).label("centered, strict"))
        .title("window anchors")
}
