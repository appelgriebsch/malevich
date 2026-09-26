{
    use malevich::{Plot, Points};
    use super::penguins;
    let p = penguins();
    Plot::new()
        .layer(Points::xy(p.flipper, p.mass).color_by(p.species))
        .title("flipper length against body mass")
        .x_label("mm")
        .y_label("g")
}
