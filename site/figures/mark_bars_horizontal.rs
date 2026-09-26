{
    use malevich::{Bars, Plot, Rule, Scale};
    // Long category names take the measured label gutter instead of a band's width.
    let workloads = ["slice a buffer", "iterate indices", "walk an adjacency list", "memo lookup", "random access"];
    Plot::new()
        .y_scale(Scale::bands(workloads))
        .layer(Bars::new(workloads, vec![1.02, 1.38, 2.11, 0.91, 1.63]).horizontal())
        .layer(Rule::v(1.0).label("baseline"))
        .title("Bars::horizontal")
        .x_label("speedup, ×")
}
