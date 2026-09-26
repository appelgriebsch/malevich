{
    use malevich::{Align, Cells, Plot, Scale, Text};
    // Bands on y label matrix rows in matrix order; text in a band aligns to its box.
    let classes = ["cat", "dog", "bird"];
    let counts = vec![42.0, 3.0, 1.0, 5.0, 37.0, 2.0, 0.0, 4.0, 29.0];
    let mut plot = Plot::new()
        .layer(Cells::matrix(3, counts.clone()))
        .x_scale(Scale::bands(classes))
        .y_scale(Scale::bands(classes));
    for (index, count) in counts.iter().enumerate() {
        plot = plot.layer(Text::at((index % 3) as f64, (index / 3) as f64, format!("{count}")).align(Align::Center));
    }
    plot.title("a confusion matrix on two band axes").x_label("predicted").y_label("actual")
}
