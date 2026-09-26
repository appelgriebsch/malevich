{
    use malevich::{Plot, Points};
    use super::penguins;
    let p = penguins();
    Plot::new().layer(Points::xy(p.bill_length, p.bill_depth).color_by(p.species))
}
