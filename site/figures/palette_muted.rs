{
    use malevich::scale::Palette;
    use malevich::{Plot, Points};
    let mut unit = super::noise(43);
    let groups = ["a", "b", "c", "d", "e", "f", "g", "h"];
    let category: Vec<&str> = (0..160).map(|i| groups[i % 8]).collect();
    let x: Vec<f64> = (0..160).map(|i| (i % 8) as f64 + unit() * 0.8).collect();
    let y: Vec<f64> = (0..160).map(|_| unit() * 10.0).collect();
    Plot::new().layer(Points::xy(x, y).color_by(category)).palette(Palette::MUTED).title("Palette::MUTED")
}
