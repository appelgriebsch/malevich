//! A histogram via the `Bin` stat: automatic Sturges/Freedman–Diaconis bin count,
//! nice decimal edges, contiguous bars from zero. Synthetic bell-ish data.

use malevich::Frame;

fn main() {
    let samples: Vec<f64> = (0..4000)
        .map(|i| {
            let i = i as f64;
            ((i * 0.731).sin() + (i * 1.13).sin() + (i * 2.71).sin()) * 2.0 + 10.0
        })
        .collect();
    let chart = malevich::hist(&samples[..]).title("distribution of a synthetic signal");
    println!("{}", chart.render(&Frame::plain(64, 15)));
}
