//! The README sample chart; spliced into README.md by `regen_docs`.

use malevich::Frame;

fn main() {
    println!(
        "{}",
        malevich::line(&[1.0, 5.0, 2.0, 8.0][..]).render(&Frame::plain(30, 8))
    );
}
