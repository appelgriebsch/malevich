//! The training-loop story: two series over shared scales, gaps where a metric was
//! not recorded. Synthetic data with a deterministic shape.

use malevich::{Frame, Line, Plot};

fn main() {
    let steps: Vec<f64> = (0..120).map(f64::from).collect();
    let train: Vec<f64> = steps
        .iter()
        .map(|s| 3.8 * (-0.035 * s).exp() + 0.32 + 0.05 * (s * 0.7).sin())
        .collect();
    // Validation runs every fourth step; the rest are gaps, drawn as breaks.
    let val: Vec<f64> = steps
        .iter()
        .map(|s| {
            if s % 4.0 == 0.0 {
                4.0 * (-0.03 * s).exp() + 0.55 + 0.08 * (s * 0.35).cos()
            } else {
                f64::NAN
            }
        })
        .collect();

    let plot = Plot::new()
        .layer(Line::xy(&steps[..], &train[..]))
        .layer(Line::xy(&steps[..], &val[..]))
        .title("loss per training step (synthetic)");
    println!("{}", plot.render(&Frame::plain(76, 18)));
}
