//! Benchmark comparisons read sideways: horizontal grouped bars, composed from
//! the grammar — `stat::dodge` places one positioned layer per series within
//! each band, `Bars::horizontal` runs the bands down the y axis so the long
//! workload names get the measured label gutter instead of a band's width, and
//! a vertical `Rule` marks the 1.0× baseline. Speedups are synthetic.
//!
//! With `--svg`, prints the same chart as an SVG terminal card — the picture a
//! README or a notebook export draws when it cannot carry escape codes.

use malevich::{Bars, Frame, Plot, Rule, Scale, Theme};

fn main() {
    let workloads = [
        "slice a buffer",
        "iterate indices",
        "walk an adjacency list",
        "memo lookup",
        "random access",
        "sort ranges",
    ];
    // Speedup over the baseline representation, per workload (synthetic).
    let packed_u64 = [1.02, 1.38, 2.11, 0.91, 1.63, 1.24];
    let packed_u32 = [1.31, 1.09, 1.72, 1.05, 2.42, 1.57];
    let width = 0.36;
    let positions = malevich::stat::dodge(&[&packed_u64, &packed_u32], width);
    let chart = Plot::new()
        .y_scale(Scale::bands(workloads))
        .layer(
            Bars::at(&positions[0][..], width, &packed_u64[..])
                .horizontal()
                .label("packed u64"),
        )
        .layer(
            Bars::at(&positions[1][..], width, &packed_u32[..])
                .horizontal()
                .label("packed u32"),
        )
        .layer(Rule::v(1.0).label("baseline"))
        .title("speedup over Range<usize> (synthetic)")
        .x_label("×");
    if std::env::args().any(|argument| argument == "--svg") {
        let frame = Frame {
            theme: Theme::DARK,
            ..Frame::portable(72, 22)
        };
        print!("{}", chart.to_svg(&frame));
    } else {
        println!("{}", chart.render(&Frame::plain(72, 22)));
    }
}
