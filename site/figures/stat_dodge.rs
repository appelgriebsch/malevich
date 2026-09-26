{
    use malevich::stat::dodge;
    use malevich::{Bars, Plot, Scale};
    let cities = ["Oslo", "Lima", "Osaka", "Cairo"];
    let morning = vec![14.0, 22.0, 9.0, 31.0];
    let evening = vec![11.0, 19.0, 13.0, 27.0];
    // dodge computes side-by-side positions within each band; Bars::at draws there.
    let width = 0.38;
    let positions = dodge(&[&morning, &evening], width);
    Plot::new()
        .x_scale(Scale::bands(cities))
        .layer(Bars::at(positions[0].clone(), width, morning).label("morning"))
        .layer(Bars::at(positions[1].clone(), width, evening).label("evening"))
        .title("stat::dodge — grouped bars")
}
