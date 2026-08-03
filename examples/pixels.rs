//! The top rung of the resolution ladder: the plot panel as a real image.
//!
//! Detects the terminal's pixel protocol and renders a plot whose chrome is
//! ordinary text while the panel is device-pixel graphics; without support it
//! falls back to cell rendering, like every other rung.
//!
//! ```sh
//! cargo run --example pixels --features pixel
//! cargo run --example pixels --features pixel -- --sixel   # force a protocol
//! ```

use malevich::pixel::{Graphics, Protocol};
use malevich::{Frame, Line, Plot, Points, Rule};

fn main() {
    let x: Vec<f64> = (0..400).map(|i| f64::from(i) * 0.025).collect();
    let wave: Vec<f64> = x
        .iter()
        .map(|v| (v * 2.0).sin() * (-v * 0.25).exp())
        .collect();
    let envelope: Vec<f64> = x.iter().map(|v| (-v * 0.25).exp()).collect();
    let samples: Vec<f64> = x.iter().step_by(20).copied().collect();
    let sampled: Vec<f64> = wave.iter().step_by(20).copied().collect();

    let plot = Plot::new()
        .layer(Line::xy(&x[..], &wave[..]).label("response"))
        .layer(Line::xy(&x[..], &envelope[..]).label("envelope"))
        .layer(Points::xy(&samples[..], &sampled[..]).label("samples"))
        .layer(Rule::h(0.0))
        .title("damped oscillator")
        .x_label("seconds");

    let frame = Frame::detect();
    let forced = std::env::args()
        .nth(1)
        .and_then(|flag| match flag.as_str() {
            "--sixel" => Some(Protocol::Sixel),
            "--kitty" => Some(Protocol::Kitty),
            "--iterm2" => Some(Protocol::ITerm2),
            _ => None,
        });
    let graphics = match forced {
        // A forced protocol keeps the detected cell size when there is one.
        Some(protocol) => Some(
            Graphics::detect()
                .unwrap_or(Graphics::new(protocol))
                .protocol(protocol),
        ),
        None => Graphics::detect(),
    };
    match graphics {
        Some(graphics) => println!("{}", plot.render_pixels(&frame, &graphics)),
        None => println!("{plot}"),
    }
}
