{
    use malevich::{Dash, Line, Plot, Rule};
    let x: Vec<f64> = (0..100).map(f64::from).collect();
    let y: Vec<f64> = x.iter().map(|x| 60.0 + 25.0 * (x * 0.08).sin() + x * 0.2).collect();
    Plot::new()
        .layer(Rule::v_span(10.0, 30.0).label("warm-up"))
        .layer(Rule::h_span(70.0, 80.0).label("tolerance"))
        .layer(Line::xy(x, y).label("signal"))
        .layer(Rule::h(60.0).dash(Dash::Dashed).label("baseline"))
        .layer(Rule::v(75.0).dash(Dash::Dotted).label("deploy"))
        .title("rules: lines and spans")
}
