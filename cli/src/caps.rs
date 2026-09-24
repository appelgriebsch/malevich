//! `kaz caps`: what detection would decide for the plot destination — the
//! frame's charset, color tier, and size, and the pixel protocols on offer
//! with the tier that answered. Reading it is the way to learn why a plot
//! came out in ASCII, or why no image appeared.

use std::io::{self, IsTerminal};

use malevich::pixel::{Capabilities, Source};

use crate::args::{Args, Output};
use crate::output;

/// The report, one `name: value` per line.
pub fn report(args: &Args) -> String {
    match &args.output {
        Output::Stdout => describe(&io::stdout(), args),
        _ => describe(&io::stderr(), args),
    }
}

fn describe<T: IsTerminal>(destination: &T, args: &Args) -> String {
    let frame = output::frame_for(destination, args);
    let capabilities = Capabilities::detect_for(destination);
    let protocols = if capabilities.protocols.is_empty() {
        "none (cells)".to_string()
    } else {
        capabilities
            .protocols
            .iter()
            .map(|protocol| format!("{protocol:?}").to_ascii_lowercase())
            .collect::<Vec<String>>()
            .join(", ")
    };
    let cell = capabilities
        .cell_size
        .map_or("unknown (8x16 assumed)".to_string(), |(w, h)| {
            format!("{w}x{h} px")
        });
    let source = match capabilities.source {
        Source::Probed => "probed (the terminal answered)",
        _ => "sniffed (environment only)",
    };
    format!(
        "destination: {}\ncharset: {}\ncolor: {}\nsize: {}x{} cells\ngraphics: {protocols}\ncell: {cell}\nsource: {source}\n",
        if destination.is_terminal() {
            "terminal"
        } else {
            "not a terminal"
        },
        format!("{:?}", frame.charset).to_ascii_lowercase(),
        format!("{:?}", frame.color).to_ascii_lowercase(),
        frame.width,
        frame.height
    )
}
