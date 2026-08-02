//! A labeled scatter: two clusters as separate layers, named in the legend.
//! Synthetic data with a deterministic shape.

use malevich::{Frame, Plot, Points};

fn main() {
    // Two deterministic blobs, jittered by incommensurate sine waves.
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
    let (ax, ay) = blob(60, 3.0, 4.0, 1.6);
    let (bx, by) = blob(60, 7.5, 7.0, 1.9);

    let plot = Plot::new()
        .layer(Points::xy(&ax[..], &ay[..]).label("colony a"))
        .layer(Points::xy(&bx[..], &by[..]).label("colony b"))
        .title("two colonies (synthetic)");
    println!("{}", plot.render(&Frame::plain(64, 18)));
}
