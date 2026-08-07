//! The README sample chart; spliced into README.md by `regen_docs`.

use malevich::{Charset, Frame};

fn main() {
    let frame = Frame {
        charset: Charset::Quadrants,
        ..Frame::plain(30, 8)
    };
    println!(
        "{}",
        malevich::line(&[1.0, 5.0, 2.0, 8.0][..]).render(&frame)
    );
}
