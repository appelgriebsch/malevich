{
    use malevich::{Bars, Plot};
    // Irregular bins: each bar between its own edges. Widths of 1, 2, 4, 8, 16.
    let starts = vec![1.0, 2.0, 4.0, 8.0, 16.0];
    let ends = vec![2.0, 4.0, 8.0, 16.0, 32.0];
    Plot::new()
        .layer(Bars::intervals(starts, ends, vec![14.0, 22.0, 31.0, 18.0, 6.0]))
        .title("Bars::intervals — bins of unequal width")
}
