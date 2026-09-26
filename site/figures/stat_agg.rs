{
    use malevich::stat::{Agg, Reducer};
    use malevich::{Bars, Plot};
    use super::penguins;
    // Group by a key, reduce each group: the median mass per species.
    let p = penguins();
    let (species, medians) = Agg::by(p.species, p.mass).reduce(Reducer::Median);
    Plot::new()
        .layer(Bars::new(species, medians))
        .title("Agg::by(...).reduce(Reducer::Median)")
        .y_label("g")
}
