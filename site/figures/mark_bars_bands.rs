{
    use malevich::{Bars, Plot};
    Plot::new()
        .layer(Bars::new(["rust", "go", "python", "typescript", "zig"], vec![68.0, 41.0, 55.0, 62.0, 12.0]))
        .title("Bars::new — one bar per band")
}
