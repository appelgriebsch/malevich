//! Violin plots: one mirrored kernel density per category, scaled to a shared
//! width — the shape of a distribution where a box shows only its summary.

use malevich::Frame;

fn main() {
    let bell = |i: f64, seed: f64| {
        ((i * 0.97 + seed).sin() + (i * 1.31 + seed * 2.0).sin() + (i * 2.63 + seed * 3.0).sin())
            / 3.0
    };
    let group = |center: f64, spread: f64, seed: f64| -> Vec<f64> {
        (0..400)
            .map(|i| center + bell(i as f64, seed) * spread)
            .collect()
    };
    let alpha = group(5.0, 2.0, 1.0);
    let beta = group(7.0, 1.2, 4.0);
    let gamma = group(4.0, 3.0, 8.0);
    let chart = malevich::violin(
        ["alpha", "beta", "gamma"],
        [&alpha[..], &beta[..], &gamma[..]],
    )
    .title("the same three groups, as densities (synthetic)");
    println!("{}", chart.render(&Frame::plain(56, 15)));
}
