{
    use malevich::scale::Unit;
    use malevich::{Bars, Plot, Scale};
    // The benchmark suite, plotted by the library it measures (BENCHMARKS.md, 2026-09-24).
    let rows = [
        "line, 10k points", "line, 10M points", "cells 2048² → 80×24", "fit, 1M pairs",
        "color_by, 5 groups", "color_by, 100k groups", "mapping, 10M", "dashboard 200×50",
        "zoom 10M, 200×50", "hover snap, 10M",
    ];
    let seconds = vec![69.3e-6, 30.7e-3, 40.6e-3, 5.13e-3, 2.02e-3, 5.35e-3, 2.08e-3, 1.77e-3, 18.4e-3, 25.4e-3];
    Plot::new()
        .y_scale(Scale::bands(rows))
        .layer(Bars::new(rows, seconds).horizontal())
        .log_x()
        .x_unit(Unit::si("s"))
        .title("the recorded baseline, Apple M1 Pro, single thread")
}
