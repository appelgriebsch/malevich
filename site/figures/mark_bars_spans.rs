{
    use malevich::{Bars, Plot};
    // Contiguous spans on a numeric axis: bin 0 starts at 10 and each is 5 wide.
    Plot::new()
        .layer(Bars::spans(10.0, 5.0, vec![2.0, 9.0, 17.0, 12.0, 6.0, 3.0, 1.0]))
        .title("Bars::spans — contiguous bins from 10, width 5")
}
