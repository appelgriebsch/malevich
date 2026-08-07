//! The README bars sample; spliced into README.md by `regen_docs`.

use malevich::{Charset, Frame};

fn main() {
    let frame = Frame {
        charset: Charset::Quadrants,
        ..Frame::plain(40, 9)
    };
    println!(
        "{}",
        malevich::bar(
            ["mon", "tue", "wed", "thu", "fri"],
            &[3.0, 7.0, 4.5, 8.0, 6.0][..]
        )
        .render(&frame)
    );
}
