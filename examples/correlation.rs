//! A correlation heatmap: signed data on a diverging colormap centered at zero,
//! so anti-correlation and correlation read as opposite colors of equal weight —
//! and the colorbar spans symmetrically. Readable at every color tier.

use malevich::Frame;
use malevich::scale::Colormap;

fn main() {
    let n = 8usize;
    let grid: Vec<f64> = (0..n * n)
        .map(|i| {
            let (row, column) = ((i / n) as f64, (i % n) as f64);
            if row == column {
                1.0
            } else {
                // Symmetric, decaying with distance, alternating in sign — the
                // shape of a real feature-correlation matrix.
                ((row - column).abs() * -0.35).exp() * ((row + column) * 0.55).cos()
            }
        })
        .collect();
    let options = malevich::HeatmapOptions::new().colormap(Colormap::RED_BLUE.centered_at(0.0));
    let chart = malevich::heatmap_with(n, &grid[..], options)
        .expect("a named colormap is valid")
        .title("correlation matrix (synthetic)");
    println!("{}", chart.render_best(&Frame::plain(46, 14)));
}
