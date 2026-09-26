{
    use malevich::{Line, Plot};
    use super::day_stamp;
    let mut unit = super::noise(31);
    // One trading session: hour labels, and the date they leave out printed once.
    let open = day_stamp(2026, 8, 3) + 9.5 * 3600.0;
    let stamps: Vec<f64> = (0..390).map(|m| open + f64::from(m) * 60.0).collect();
    let mut price = 184.0;
    let prices: Vec<f64> = stamps.iter().map(|_| { price += super::gaussian(&mut unit) * 0.12; price }).collect();
    Plot::new().layer(Line::xy(stamps, prices)).time_x().title("a session, minute by minute").y_label("$")
}
