//! `kaz spec`: render a serialized malevich `Document` — the JSON a plot or
//! grid round-trips through serde — so a spec produced anywhere (a notebook,
//! another language's rim, a file checked into a repo) draws in the terminal
//! through the same destination and detection path as every chart.

use malevich::Document;

use crate::args::{Args, Fail};
use crate::chart::Built;
use crate::output;
use crate::recipe::Furniture;

/// Parses `raw` as a document and writes it to the plot destination. Plot
/// documents take the invocation's furniture on top; grid documents render as
/// they are.
pub fn run(args: &Args, raw: &str) -> Result<i32, Fail> {
    let document: Document =
        serde_json::from_str(raw).map_err(|error| Fail(format!("spec: {error}")))?;
    let outcome = if let Some(plot) = document.as_plot() {
        let built = Built {
            plot: Furniture::from_args(args).apply(plot.clone()),
            unparsed: 0,
        };
        output::emit(args, &built)
    } else if let Some(grid) = document.as_grid() {
        let frame = output::frame_for_output(args);
        match grid.try_render(&frame) {
            Ok(text) => output::emit_text(args, &text),
            Err(error) => Err(output::EmitError::Render(error)),
        }
    } else {
        return Err(Fail(
            "spec: the document holds neither a plot nor a grid".into(),
        ));
    };
    match outcome {
        Ok(code) => Ok(code),
        Err(output::EmitError::Io(error)) if error.kind() == std::io::ErrorKind::BrokenPipe => {
            Ok(0)
        }
        Err(output::EmitError::Io(error)) => Err(Fail(format!("write failed: {error}"))),
        Err(output::EmitError::Render(error)) => Err(Fail(format!("render failed: {error}"))),
    }
}
