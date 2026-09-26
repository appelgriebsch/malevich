{
    use malevich::{Bars, Plot};
    // A per-bar base is the y2 channel: the second layer starts where the first ends.
    let quarters = ["Q1", "Q2", "Q3", "Q4"];
    let hardware = vec![12.0, 15.0, 11.0, 19.0];
    let software = [8.0, 9.5, 14.0, 12.0];
    Plot::new()
        .layer(Bars::new(quarters, hardware.clone()).label("hardware"))
        .layer(Bars::new(quarters, software.iter().zip(&hardware).map(|(s, h)| s + h).collect::<Vec<_>>()).base(hardware).label("software"))
        .title("stacked through Bars::base")
}
