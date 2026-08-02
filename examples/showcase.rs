//! A colored tour of the current marks, rendered for *your* terminal.
//!
//! Uses `Frame::detect()`: charts size themselves to the terminal width, use color
//! when the terminal has any, and degrade to plain text when piped. This example is
//! deliberately not part of the deterministic gallery — its output depends on where
//! you run it, which is the point.

use malevich::{Frame, Line, Plot, Points};

fn main() {
    let frame = Frame::detect();

    let steps: Vec<f64> = (0..120).map(f64::from).collect();
    let train: Vec<f64> = steps
        .iter()
        .map(|s| 3.8 * (-0.035 * s).exp() + 0.32 + 0.05 * (s * 0.7).sin())
        .collect();
    let val: Vec<f64> = steps
        .iter()
        .map(|s| 4.0 * (-0.03 * s).exp() + 0.55 + 0.08 * (s * 0.35).cos())
        .collect();
    println!(
        "{}\n",
        Plot::new()
            .layer(Line::xy(&steps[..], &train[..]).label("train"))
            .layer(Line::xy(&steps[..], &val[..]).label("val"))
            .title("loss (synthetic)")
            .render(&frame)
    );

    println!(
        "{}\n",
        malevich::bar(
            ["rust", "go", "python", "typescript", "zig"],
            &[68.0, 41.0, 55.0, 62.0, 12.0][..],
        )
        .title("admired languages, % (synthetic)")
        .render(&frame)
    );

    let blob = |n: usize, cx: f64, cy: f64, spread: f64| -> (Vec<f64>, Vec<f64>) {
        (0..n)
            .map(|i| {
                let i = i as f64;
                (
                    cx + spread * (i * 0.97).sin() * (i * 0.31).cos(),
                    cy + spread * 0.6 * (i * 1.13).cos() * (i * 0.47).sin(),
                )
            })
            .unzip()
    };
    let (ax, ay) = blob(80, 3.0, 4.0, 1.6);
    let (bx, by) = blob(80, 7.5, 7.0, 1.9);
    println!(
        "{}\n",
        Plot::new()
            .layer(Points::xy(&ax[..], &ay[..]).label("colony a"))
            .layer(Points::xy(&bx[..], &by[..]).label("colony b"))
            .title("two colonies (synthetic)")
            .render(&frame)
    );

    println!(
        "{}",
        Plot::new()
            .layer(Line::function(0.0..12.6, f64::sin).label("sin"))
            .layer(Line::function(0.0..12.6, |x| (x * 0.5).cos() * 0.6).label("cos/2"))
            .title("function sampling")
            .render(&frame)
    );
}
