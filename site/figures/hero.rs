{
    use malevich::{Area, Color, Dash, Line, Plot, Rule, Text};

    let steps: Vec<f64> = (0..120).map(f64::from).collect();
    let train: Vec<f64> = steps
        .iter()
        .map(|s| 3.8 * (-0.035 * s).exp() + 0.32 + 0.05 * (s * 0.7).sin())
        .collect();
    let val: Vec<f64> = steps
        .iter()
        .map(|s| 4.0 * (-0.03 * s).exp() + 0.55 + 0.08 * (s * 0.35).cos())
        .collect();
    Plot::new()
        .layer(Area::xy(steps.clone(), train.clone()).color(Color::Rgb(8, 52, 52)))
        .layer(Line::xy(steps.clone(), train).label("train"))
        .layer(Line::xy(steps, val).label("val").dash(Dash::Dotted))
        .layer(Rule::h(0.5).label("target").dash(Dash::Dashed))
        .layer(Text::at(62.0, 2.1, "< converging"))
        .title("loss")
        .x_label("step")
        .y_label("loss")
}
