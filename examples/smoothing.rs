//! The Window stat: a rolling mean laid over its noisy source.

use malevich::{Frame, Line, Plot};

fn main() {
    let raw: Vec<f64> = (0..120)
        .map(|i| 3.0 * (-0.03 * i as f64).exp() + 0.4 + ((i * 7) % 13) as f64 * 0.06)
        .collect();
    let smooth = malevich::stat::Window::new(9).mean(&raw);
    let chart = Plot::new()
        .layer(Line::y(&raw[..]).label("raw"))
        .layer(Line::y(&smooth[..]).label("rolling mean, k = 9"))
        .title("smoothing a noisy series (synthetic)");
    println!("{}", chart.render(&Frame::plain(64, 14)));
}
