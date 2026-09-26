{
    use malevich::{Line, Plot};
    use super::co2;
    // Sixty-seven years of monthly CO₂: year labels at a tick step the span demands.
    let (stamps, ppm) = co2();
    Plot::new().layer(Line::xy(stamps, ppm)).time_x().title("CO₂ at Mauna Loa (NOAA)").y_label("ppm")
}
