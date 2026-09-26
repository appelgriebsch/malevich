{
    use malevich::stat::ewma;
    use malevich::{Line, Plot};
    use super::loss_log;
    // A real training log, and its debiased exponential moving average.
    let loss = loss_log();
    Plot::new()
        .layer(Line::y(loss.clone()).label("per-step loss"))
        .layer(Line::y(ewma(&loss, 0.05)).label("ewma α = 0.05").glow())
        .title("topos bigram model, 1,000 steps")
        .x_label("step")
        .y_label("loss")
}
