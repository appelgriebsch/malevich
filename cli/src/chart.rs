//! Subcommand → [`Plot`]. Zero rendering logic: every chart is a preset or the
//! exact public grammar composition the preset is proven equal to (D-C3).

use malevich::{Line, Plot, Points};

use crate::args::{Args, Command};
use crate::input::Table;
use crate::series::{self, Series};

/// A built plot plus the count of fields that would not parse.
pub struct Built {
    pub plot: Plot<'static>,
    pub unparsed: usize,
}

/// Builds the plot for `args` over `table`.
pub fn build(args: &Args, table: &Table) -> Built {
    let (plot, unparsed) = match args.command {
        Command::Line => value_plot(args, table, Kind::Line),
        Command::Scatter => value_plot(args, table, Kind::Scatter),
        Command::Hist => hist_plot(table, args.bins),
        Command::Bar => bar_plot(table),
        Command::Count => count_plot(table),
        Command::Density => distribution(table, malevich::density),
        Command::Ecdf => distribution(table, malevich::ecdf),
        Command::Box => box_plot(table),
        Command::Violin => violin_plot(table),
        Command::Hist2d => hist2d_plot(args, table),
        Command::Heatmap => heatmap_plot(table),
    };
    Built {
        plot: furniture(plot, args),
        unparsed,
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Line,
    Scatter,
}

/// Line and scatter: one layer per series, following `--fmt`.
fn value_plot(args: &Args, table: &Table, kind: Kind) -> (Plot<'static>, usize) {
    let fmt = series::resolve_fmt(table, args.fmt);
    let data = series::dataset(table, fmt, args.time_x);
    let mut plot = Plot::new();
    for series in data.series {
        plot = layer(plot, series, kind);
    }
    (plot, data.unparsed)
}

/// Adds one series as a line or a scatter layer, labeled when named.
fn layer(plot: Plot<'static>, series: Series, kind: Kind) -> Plot<'static> {
    let Series { x, y, label } = series;
    match (kind, x) {
        (Kind::Line, Some(x)) => plot.layer(named(Line::xy(x, y), label, Line::label)),
        (Kind::Line, None) => plot.layer(named(Line::y(y), label, Line::label)),
        (Kind::Scatter, Some(x)) => plot.layer(named(Points::xy(x, y), label, Points::label)),
        (Kind::Scatter, None) => plot.layer(named(Points::y(y), label, Points::label)),
    }
}

/// Applies a label to a mark when one is present, via the mark's own setter.
fn named<M>(mark: M, label: Option<String>, set: impl FnOnce(M, String) -> M) -> M {
    match label {
        Some(text) => set(mark, text),
        None => mark,
    }
}

/// Histogram: pool every numeric field, then bin. Auto by default; with `--bins N`
/// the exact documented expansion of `hist` — `stat::Bins::new` + `Bars::spans`.
fn hist_plot(table: &Table, bins: Option<usize>) -> (Plot<'static>, usize) {
    let (values, unparsed) = series::flatten(table);
    let plot = match bins {
        None => malevich::hist(values),
        Some(count) => binned(&values, count),
    };
    (plot, unparsed)
}

/// The `--bins N` expansion: `count` equal-width bins over the finite data range,
/// counted into a `Bars::spans` layer — exactly what `hist` does, minus the
/// automatic bin-count choice.
fn binned(values: &[f64], count: usize) -> Plot<'static> {
    use malevich::Bars;
    use malevich::stat::Bins;

    let (min, max) = values
        .iter()
        .filter(|value| value.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &value| {
            (lo.min(value), hi.max(value))
        });
    if !min.is_finite() {
        return Plot::new();
    }
    // A degenerate range still yields one honest bin around the value.
    let (start, width) = if min == max {
        (min - 0.5, 1.0)
    } else {
        (min, (max - min) / count as f64)
    };
    let mut histogram = Bins::new(start, width, count);
    for &value in values {
        histogram.add(value);
    }
    let counts: Vec<f64> = histogram.counts().iter().map(|&c| c as f64).collect();
    Plot::new().layer(Bars::spans(histogram.start(), histogram.width(), counts))
}

/// Density and ecdf: pool every numeric field, then the matching preset.
fn distribution(table: &Table, preset: fn(Vec<f64>) -> Plot<'static>) -> (Plot<'static>, usize) {
    let (values, unparsed) = series::flatten(table);
    (preset(values), unparsed)
}

/// Box plots: each column a group (header names, else positions).
fn box_plot(table: &Table) -> (Plot<'static>, usize) {
    let (categories, groups, unparsed) = series::groups(table);
    (malevich::box_plot(categories, groups), unparsed)
}

/// Violin plots: same column-as-group shape as box.
fn violin_plot(table: &Table) -> (Plot<'static>, usize) {
    let (categories, groups, unparsed) = series::groups(table);
    (malevich::violin(categories, groups), unparsed)
}

/// 2D histogram: the first two columns as x and y (x is time under `--time-x`).
fn hist2d_plot(args: &Args, table: &Table) -> (Plot<'static>, usize) {
    let (x, y, unparsed) = series::xy(table, args.time_x);
    (malevich::hist2d(x, y), unparsed)
}

/// Heatmap: the rows as a row-major grid (first line on top).
fn heatmap_plot(table: &Table) -> (Plot<'static>, usize) {
    let (columns, values, unparsed) = series::matrix(table);
    if columns == 0 {
        return (Plot::new(), unparsed);
    }
    (malevich::heatmap(columns, values), unparsed)
}

/// Bar: `label value` rows straight into the `bar` preset.
fn bar_plot(table: &Table) -> (Plot<'static>, usize) {
    let (labels, values, unparsed) = series::labeled_values(table);
    (malevich::bar(labels, values), unparsed)
}

/// Count: value frequencies (CLI-side) rendered as bars. Categories are never
/// "unparseable" — every string is a valid label — so the tally is zero.
fn count_plot(table: &Table) -> (Plot<'static>, usize) {
    let (labels, values): (Vec<String>, Vec<f64>) = series::counts(table).into_iter().unzip();
    (malevich::bar(labels, values), 0)
}

/// Applies the shared furniture flags: title, axis labels, limits, log scales.
fn furniture(mut plot: Plot<'static>, args: &Args) -> Plot<'static> {
    if let Some(title) = &args.title {
        plot = plot.title(title);
    }
    if let Some(xlabel) = &args.xlabel {
        plot = plot.x_label(xlabel);
    }
    if let Some(ylabel) = &args.ylabel {
        plot = plot.y_label(ylabel);
    }
    if let Some((lo, hi)) = args.xlim {
        plot = plot.x_domain(lo, hi);
    }
    if let Some((lo, hi)) = args.ylim {
        plot = plot.y_domain(lo, hi);
    }
    // Parsing rejects --time-x on charts without a time axis, so no gate here.
    if args.time_x {
        plot = plot.time_x();
    }
    if args.log_x {
        plot = plot.log_x();
    }
    if args.log_y {
        plot = plot.log_y();
    }
    plot
}
