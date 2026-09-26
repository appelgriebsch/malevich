{
    use malevich::{Bars, Plot, Scale};
    // The same counts on Scale::Integer: the tick step never drops below one.
    Plot::new().layer(Bars::new(["a", "b", "c", "d"], vec![1.0, 3.0, 2.0, 3.0])).y_scale(Scale::Integer).title("Scale::Integer")
}
