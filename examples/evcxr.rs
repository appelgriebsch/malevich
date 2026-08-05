//! Evcxr/Jupyter HTML output as an inspectable file.
//!
//! ```sh
//! cargo run --example evcxr --features evcxr > plot.html
//! ```

use malevich::{Frame, Line, Plot, Points, Rule};

fn main() {
    let x: Vec<f64> = (0..160).map(|i| f64::from(i) * 0.05).collect();
    let signal: Vec<f64> = x
        .iter()
        .map(|value| value.sin() * (-value * 0.12).exp())
        .collect();
    let samples: Vec<f64> = x.iter().step_by(16).copied().collect();
    let sampled: Vec<f64> = signal.iter().step_by(16).copied().collect();

    let plot = Plot::new()
        .layer(Line::xy(&x[..], &signal[..]).label("signal"))
        .layer(Points::xy(&samples[..], &sampled[..]).label("samples"))
        .layer(Rule::h(0.0))
        .title("Evcxr notebook")
        .x_label("time");

    println!("{}", plot.to_html(&Frame::plain(80, 18)));
}
