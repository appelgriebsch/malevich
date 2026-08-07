//! The asciichart homage: `LineStyle::Corners` draws one box-drawing glyph per
//! column — the instantly-legible low-fi style that kroitor/asciichart made famous
//! (credited in ACKNOWLEDGEMENTS.md), here with real axes underneath it.

use malevich::{Charset, Frame, Line, LineStyle, Plot};

fn main() {
    let values: Vec<f64> = (0..60)
        .map(|i| 15.0 * (i as f64 * std::f64::consts::PI / 30.0).sin())
        .collect();
    let chart = Plot::new()
        .layer(Line::y(&values[..]).style(LineStyle::Corners))
        .title("the corners style");
    let frame = Frame {
        charset: Charset::Quadrants,
        ..Frame::plain(70, 16)
    };
    println!("{}", chart.render(&frame));
}
