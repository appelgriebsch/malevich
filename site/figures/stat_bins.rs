{
    use malevich::stat::Bins;
    use malevich::{Bars, Plot, Scale};
    use super::penguins;
    // The histogram, spelled out: Bins::auto chooses the geometry, Bars::spans draws the counts.
    let mass = penguins().mass;
    let mut bins = Bins::auto(&mass, 60).expect("finite data");
    for value in &mass {
        bins.add(*value);
    }
    let counts: Vec<f64> = bins.counts().iter().map(|&c| c as f64).collect();
    Plot::new()
        .layer(Bars::spans(bins.start(), bins.width(), counts))
        .y_scale(Scale::Integer)
        .title("Bins::auto + Bars::spans + Scale::Integer == hist")
        .x_label("body mass, g")
}
