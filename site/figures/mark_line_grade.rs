{
    use malevich::scale::Colormap;
    use malevich::{Line, Plot};
    // `grade` colors the line by a third series through a colormap: here the
    // temperature of a run colors its altitude profile.
    let km: Vec<f64> = (0..120).map(|i| f64::from(i) * 0.1).collect();
    let altitude: Vec<f64> = km.iter().map(|k| 300.0 + 180.0 * (k * 0.6).sin() + 40.0 * (k * 2.3).cos()).collect();
    let temperature: Vec<f64> = km.iter().map(|k| 12.0 + 9.0 * (k * 0.35).sin()).collect();
    Plot::new()
        .layer(Line::xy(km, altitude).grade(temperature, Colormap::MAGMA).glow())
        .colorbar()
        .title("altitude, colored by temperature")
        .x_label("km")
        .y_label("m")
}
