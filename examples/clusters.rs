//! Palmer penguins (CC0 — see examples/data/README.md): bill dimensions separate
//! the species into visible clusters, with marker shapes preserving the distinction
//! in colorless output.

use malevich::{Frame, Plot, PointStyle, Points};

fn main() {
    let mut species: [(&str, Vec<f64>, Vec<f64>); 3] = [
        ("Adelie", Vec::new(), Vec::new()),
        ("Gentoo", Vec::new(), Vec::new()),
        ("Chinstrap", Vec::new(), Vec::new()),
    ];
    for line in include_str!("data/penguins.csv").lines().skip(1) {
        let mut parts = line.split(',');
        let name = parts.next().unwrap_or_default();
        let length: f64 = parts
            .next()
            .and_then(|v| v.parse().ok())
            .unwrap_or(f64::NAN);
        let depth: f64 = parts
            .next()
            .and_then(|v| v.parse().ok())
            .unwrap_or(f64::NAN);
        if let Some((_, xs, ys)) = species.iter_mut().find(|(s, ..)| *s == name) {
            xs.push(length);
            ys.push(depth);
        }
    }
    let mut plot = Plot::new()
        .title("penguin bills by species")
        .x_label("bill length, mm")
        .y_label("depth");
    let styles = [PointStyle::Dot, PointStyle::Plus, PointStyle::Cross];
    for ((name, xs, ys), style) in species.iter().zip(styles) {
        plot = plot.layer(Points::xy(&xs[..], &ys[..]).style(style).label(*name));
    }
    println!("{}", plot.render_best(&Frame::plain(72, 20)));
}
