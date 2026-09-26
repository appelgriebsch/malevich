{
    use malevich::stat::kde;
    use malevich::{Area, Plot};
    use super::penguins;
    // A horizontal area is a fill along y: the two halves of a violin are one each.
    let flippers = penguins().of("Gentoo", &penguins().flipper);
    let (ys, density) = kde(&flippers, 120).expect("enough data");
    let left: Vec<f64> = density.iter().map(|d| -d).collect();
    Plot::new()
        .layer(Area::horizontal(ys.clone(), vec![0.0; ys.len()], density).label("Area::horizontal"))
        .layer(Area::horizontal(ys.clone(), left, vec![0.0; ys.len()]))
        .title("a violin is two horizontal areas")
        .y_label("flipper, mm")
}
