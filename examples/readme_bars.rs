//! The README bars sample; spliced into README.md by `regen_docs`.

use malevich::Frame;

fn main() {
    println!(
        "{}",
        malevich::bar(
            ["mon", "tue", "wed", "thu", "fri"],
            &[3.0, 7.0, 4.5, 8.0, 6.0][..]
        )
        .render(&Frame::plain(40, 9))
    );
}
