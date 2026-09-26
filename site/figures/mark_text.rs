{
    use malevich::{Align, Bars, Plot, Text};
    // Text at data coordinates; on a band axis, `align` sets it in the band's
    // own box, so labels and annotations land in lockstep.
    let cities = ["Oslo", "Lima", "Osaka", "Cairo"];
    let values = vec![14.0, 22.0, 9.0, 31.0];
    let mut plot = Plot::new().layer(Bars::new(cities, values.clone()));
    for (index, value) in values.iter().enumerate() {
        plot = plot.layer(Text::at(index as f64, value + 2.5, format!("{value}")).align(Align::Center));
    }
    plot.layer(Text::at(1.0, 34.0, "Text::at, aligned to its band")).y_max(38.0).title("annotated bars")
}
