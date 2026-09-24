//! Annotations: a Rule for the target line, a Rule span for the warm-up
//! phase, a Text note at data coordinates — all extend the axis domains so
//! they are never silently off-plot, and the span is a wash the line shows
//! through.

use malevich::{Frame, Line, Plot, Rule, Text};

fn main() {
    let loss: Vec<f64> = (0..70)
        .map(|i| 4.0 * (-0.06 * i as f64).exp() + 0.4)
        .collect();
    let plot = Plot::new()
        .layer(Rule::v_span(0.0, 10.0).label("warm-up"))
        .layer(Line::y(&loss[..]).label("loss"))
        .layer(Rule::h(0.5).label("target"))
        .layer(Text::at(34.0, 2.2, "< converging"))
        .title("annotated loss (synthetic)");
    println!("{}", plot.render_best(&Frame::plain(60, 14)));
}
