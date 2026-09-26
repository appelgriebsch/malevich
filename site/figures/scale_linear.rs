{
    use malevich::{Bars, Plot};
    // Counts on a tall frame: a linear axis happily labels 0.5 and 1.5.
    Plot::new().layer(Bars::new(["a", "b", "c", "d"], vec![1.0, 3.0, 2.0, 3.0])).title("Scale::Linear")
}
