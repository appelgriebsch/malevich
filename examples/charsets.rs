//! The charset ladder: one curve at every subpixel density, from solid blocks to
//! braille dots down to plain ASCII. `Frame::detect` picks the densest your terminal
//! and font are known to render; here they are side by side so the trade-off is
//! visible. Sextants (Unicode 13) and octants (Unicode 16) need a recent font — on an
//! older one their glyphs show as tofu, which is exactly why detection is careful.

use malevich::{Charset, Frame, Line, Plot};

fn main() {
    let x: Vec<f64> = (0..90).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin() * (v * 0.5).cos()).collect();
    for (charset, label) in [
        (
            Charset::Octants,
            "Octants — 2x4 solid blocks (Unicode 16, densest ink)",
        ),
        (
            Charset::Sextants,
            "Sextants — 2x3 solid blocks (Unicode 13)",
        ),
        (
            Charset::Quadrants,
            "Quadrants — 2x2 solid blocks (renders anywhere)",
        ),
        (Charset::HalfBlocks, "Half blocks — 1x2"),
        (Charset::Braille, "Braille — 2x4 dots (the default)"),
        (Charset::Ascii, "ASCII — 1x1, the universal fallback"),
    ] {
        let frame = Frame {
            charset,
            ..Frame::plain(60, 8)
        };
        println!("{label}");
        println!(
            "{}\n",
            Plot::new().layer(Line::xy(&x[..], &y[..])).render(&frame)
        );
    }
}
