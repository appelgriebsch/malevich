{
    use malevich::{Line, LineStyle, Plot, Rule};
    let loss = vec![4.0, 2.8, 1.9, 1.2, 0.8, 0.6, 0.55, 0.5, 0.48];
    Plot::new()
        .layer(Line::y(loss).label("loss").style(LineStyle::Corners))
        .layer(Rule::h(0.5).label("target"))
        .title("training")
        .x_label("epoch")
}
