//! A heatmap via the Cells mark: values render as a shade ramp colored by the
//! default colormap — readable at every color tier, including plain text.

use malevich::Frame;

fn main() {
    let n = 8usize;
    let grid: Vec<f64> = (0..n * n)
        .map(|i| {
            let (row, column) = ((i / n) as f64, (i % n) as f64);
            if row == column {
                1.0
            } else {
                ((row - column).abs() * -0.4).exp() * (1.0 + (row * column * 0.3).sin() * 0.1)
            }
        })
        .collect();
    let chart = malevich::heatmap(n, &grid[..]).title("correlation matrix (synthetic)");
    println!("{}", chart.render(&Frame::plain(46, 14)));
}
