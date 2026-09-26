{
    use malevich::{Bars, Plot, Rule, Scale};
    // Horizontal bars around zero, the largest swing first.
    let factors = ["price", "volume", "fx", "cost", "mix"];
    let swing = vec![-4.0, 3.5, -1.2, 0.8, 0.3];
    Plot::new()
        .y_scale(Scale::bands(factors))
        .layer(Bars::new(factors, swing).horizontal())
        .layer(Rule::v(0.0))
        .title("a tornado")
        .x_label("impact, M$")
}
