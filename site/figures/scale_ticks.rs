{
    use malevich::{Line, Plot};
    // Steps of 0.2 have no exact binary form. Every label is an exact decimal
    // regardless: an integer mantissa times a power of ten, formatted so it
    // parses back to the tick's value.
    Plot::new().layer(Line::function(0.0..60.0, |x| 0.3 + 0.3 * (x * 0.14).sin())).title("labels that parse back to their ticks")
}
