//! One normalized chart description shared by rendering and `--emit-code`.
//!
//! Input framing still belongs to [`crate::input`], but every semantic choice
//! after that boundary happens here exactly once: projection, grouping, column
//! roles, parsing, histogram geometry, furniture, and frame size. Backends only
//! translate this value into a retained plot or Rust source.

use std::fmt;

use malevich::scale::{Colormap, Unit};
use malevich::stat::Normalization;

use crate::args::{Args, BarLayout, Command};
use crate::input::{self, Table};
use crate::series::{self, Dataset};

/// A chart whose input semantics have already been resolved.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Recipe {
    pub command: Command,
    pub chart: Chart,
    pub furniture: Furniture,
    pub frame: FrameSize,
    pub unparsed: usize,
}

/// The small set of data shapes the CLI maps onto malevich's public grammar.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Chart {
    Value {
        mark: ValueMark,
        data: Dataset,
    },
    ScatterBy {
        x: Vec<f64>,
        y: Vec<f64>,
        groups: Vec<String>,
    },
    Histogram {
        start: f64,
        width: f64,
        heights: Vec<f64>,
        normalization: Normalization,
    },
    Bars {
        labels: Vec<String>,
        values: Vec<f64>,
        horizontal: bool,
    },
    BarGroups {
        labels: Vec<String>,
        names: Vec<String>,
        series: Vec<Vec<f64>>,
        layout: BarLayout,
        horizontal: bool,
    },
    Describe {
        names: Vec<String>,
        groups: Vec<Vec<f64>>,
    },
    Table {
        rows: Vec<String>,
        columns: Vec<String>,
        values: Vec<f64>,
    },
    Distribution {
        kind: DistributionKind,
        values: Vec<f64>,
    },
    Spark {
        values: Vec<f64>,
    },
    Grouped {
        kind: GroupedKind,
        categories: Vec<String>,
        groups: Vec<Vec<f64>>,
    },
    Grid {
        columns: usize,
        values: Vec<f64>,
        extents: Option<((f64, f64), (f64, f64))>,
        colormap: Colormap,
        labels_x: Option<Vec<String>>,
        labels_y: Option<Vec<String>>,
        reduce: Option<malevich::stat::Reducer>,
    },
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueMark {
    Line,
    Scatter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DistributionKind {
    Density,
    Ecdf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GroupedKind {
    Box,
    Violin,
}

/// Target-independent plot furniture.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Furniture {
    pub title: Option<String>,
    pub xlabel: Option<String>,
    pub ylabel: Option<String>,
    pub xlim: Option<(f64, f64)>,
    pub ylim: Option<(f64, f64)>,
    pub time_x: bool,
    pub log_x: bool,
    pub log_y: bool,
    /// The unit the value axis's labels carry.
    pub unit: Option<Unit>,
    /// Whether the value axis is x (sideways bars), so the unit goes there.
    pub unit_on_x: bool,
    /// Horizontal reference lines, drawn after the chart's own layers.
    pub hlines: Vec<f64>,
    /// Vertical reference lines.
    pub vlines: Vec<f64>,
}

impl Furniture {
    /// The furniture an invocation asks for, chart-independent.
    pub(crate) fn from_args(args: &Args) -> Furniture {
        Furniture {
            title: args.title.clone(),
            xlabel: args.xlabel.clone(),
            ylabel: args.ylabel.clone(),
            xlim: args.xlim,
            ylim: args.ylim,
            time_x: args.time_x,
            log_x: args.log_x,
            log_y: args.log_y,
            unit: args.unit.clone(),
            unit_on_x: args.horizontal && args.command == Command::Bar,
            hlines: args.hlines.clone(),
            vlines: args.vlines.clone(),
        }
    }
}

/// Dimensions captured for generated programs. Runtime output still applies
/// charset and destination detection around these overrides.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct FrameSize {
    pub width: Option<usize>,
    pub height: Option<usize>,
}

/// Failure while normalizing raw table input into a recipe.
#[derive(Debug)]
pub(crate) enum PrepareError {
    Input(String),
    Chart(malevich::Error),
}

impl fmt::Display for PrepareError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrepareError::Input(message) => formatter.write_str(message),
            PrepareError::Chart(error) => write!(formatter, "chart: {error}"),
        }
    }
}

impl From<malevich::Error> for PrepareError {
    fn from(error: malevich::Error) -> PrepareError {
        PrepareError::Chart(error)
    }
}

/// Applies selectors and compiles a framed table into one backend-neutral chart.
pub(crate) fn prepare(args: &Args, mut table: Table) -> Result<Recipe, PrepareError> {
    if let Some(selectors) = &args.cols {
        table = input::select(&table, selectors).map_err(PrepareError::Input)?;
    }

    let categories = if let Some(selector) = &args.by {
        let index = input::column_index(&table, selector).map_err(PrepareError::Input)?;
        let categories = input::string_column(&table, index);
        let keep: Vec<String> = (0..table.width())
            .filter(|&column| column != index)
            .map(|column| column.to_string())
            .collect();
        table = input::select(&table, &keep).map_err(PrepareError::Input)?;
        Some(categories)
    } else {
        None
    };

    let (chart, unparsed) = match (args.command, categories) {
        (Command::Scatter, Some(groups)) => {
            let (x, y, unparsed) = series::xy(&table, args.time_x);
            (Chart::ScatterBy { x, y, groups }, unparsed)
        }
        (Command::Line, _) => value(&table, args, ValueMark::Line),
        (Command::Scatter, _) => value(&table, args, ValueMark::Scatter),
        (Command::Hist, _) => histogram(&table, args)?,
        (Command::Bar, _) if args.bar_layout == BarLayout::Single => {
            let (labels, values, unparsed) = series::labeled_values(&table);
            (
                Chart::Bars {
                    labels,
                    values,
                    horizontal: args.horizontal,
                },
                unparsed,
            )
        }
        (Command::Bar, _) => {
            let (labels, names, series, unparsed) = series::labeled_series(&table);
            (
                Chart::BarGroups {
                    labels,
                    names,
                    series,
                    layout: args.bar_layout,
                    horizontal: args.horizontal,
                },
                unparsed,
            )
        }
        (Command::Count, _) => {
            let (labels, values) = series::counts(&table).into_iter().unzip();
            (
                Chart::Bars {
                    labels,
                    values,
                    horizontal: false,
                },
                0,
            )
        }
        (Command::Describe, _) => {
            let (names, groups, unparsed) = series::groups(&table);
            (Chart::Describe { names, groups }, unparsed)
        }
        (Command::Table, _) => {
            let (rows, columns, values, unparsed) = series::table_cells(&table);
            let chart = if rows.is_empty() || columns.is_empty() {
                Chart::Empty
            } else {
                Chart::Table {
                    rows,
                    columns,
                    values,
                }
            };
            (chart, unparsed)
        }
        // Both are answered before any input is framed; see `main`.
        (Command::Caps | Command::Spec, _) => (Chart::Empty, 0),
        (Command::Density, _) => distribution(&table, DistributionKind::Density),
        (Command::Spark, _) => {
            let (values, unparsed) = series::flatten(&table);
            (Chart::Spark { values }, unparsed)
        }
        (Command::Ecdf, _) => distribution(&table, DistributionKind::Ecdf),
        (Command::Box, _) => grouped(&table, GroupedKind::Box),
        (Command::Violin, _) => grouped(&table, GroupedKind::Violin),
        (Command::Hist2d, _) => {
            let (x, y, unparsed) = series::xy(&table, args.time_x);
            let options = malevich::Histogram2dOptions::default();
            let chart = match malevich::stat::try_bins2(&x, &y, options.columns, options.rows)? {
                Some(grid) => Chart::Grid {
                    columns: grid.columns,
                    values: grid
                        .counts
                        .into_iter()
                        .map(|count| if count == 0.0 { f64::NAN } else { count })
                        .collect(),
                    extents: Some((grid.x, grid.y)),
                    colormap: args.colormap.clone().unwrap_or(options.colormap),
                    labels_x: None,
                    labels_y: None,
                    reduce: None,
                },
                None => Chart::Empty,
            };
            (chart, unparsed)
        }
        (Command::Heatmap, _) => {
            let (columns, values, unparsed) = series::matrix(&table, args.labels_y.is_none());
            let chart = if columns == 0 {
                Chart::Empty
            } else {
                // Band labels must match the grid before anything renders; the
                // error names the mismatch instead of quietly dropping rows.
                if let Some(labels) = &args.labels_x
                    && labels.len() != columns
                {
                    return Err(PrepareError::Input(format!(
                        "--labels-x names {} columns, but the matrix has {columns}",
                        labels.len()
                    )));
                }
                let rows = values.len() / columns;
                if let Some(labels) = &args.labels_y
                    && labels.len() != rows
                {
                    return Err(PrepareError::Input(format!(
                        "--labels-y names {} rows, but the matrix has {rows}",
                        labels.len()
                    )));
                }
                Chart::Grid {
                    columns,
                    values,
                    extents: None,
                    colormap: args
                        .colormap
                        .clone()
                        .unwrap_or(malevich::scale::Colormap::DEFAULT),
                    labels_x: args.labels_x.clone(),
                    labels_y: args.labels_y.clone(),
                    reduce: args.reduce,
                }
            };
            (chart, unparsed)
        }
    };

    Ok(Recipe {
        command: args.command,
        chart,
        furniture: Furniture::from_args(args),
        frame: FrameSize {
            width: args.width,
            height: args.height,
        },
        unparsed,
    })
}

fn value(table: &Table, args: &Args, mark: ValueMark) -> (Chart, usize) {
    let fmt = series::resolve_fmt(table, args.fmt);
    let data = series::dataset(table, fmt, args.time_x);
    let unparsed = data.unparsed;
    (Chart::Value { mark, data }, unparsed)
}

/// Resolves both automatic and explicit bins once, and their heights through
/// the same `Bins::heights` the `hist` preset uses. Keeping only bar geometry
/// gives rendering and generated source exactly the same histogram.
fn histogram(table: &Table, args: &Args) -> Result<(Chart, usize), malevich::Error> {
    use malevich::stat::Bins;

    let (values, unparsed) = series::flatten(table);
    let bins = match (args.bins, args.binwidth) {
        (Some(count), _) => Bins::try_uniform(&values, count)?,
        (None, Some(width)) => bins_of_width(&values, width)?,
        (None, None) => Bins::try_auto(&values, malevich::HistogramOptions::default().max_bins)?,
    };
    let chart = match bins {
        Some(bins) => Chart::Histogram {
            start: bins.start(),
            width: bins.width(),
            heights: bins.heights(args.normalize, args.cumulative),
            normalization: args.normalize,
        },
        None => Chart::Empty,
    };
    Ok((chart, unparsed))
}

/// Bins of a fixed `width` covering the finite values: the first edge is the
/// multiple of the width at or below the minimum, and the maximum lands in
/// the last bin (its right edge is inclusive). `None` without finite values.
fn bins_of_width(
    values: &[f64],
    width: f64,
) -> Result<Option<malevich::stat::Bins>, malevich::Error> {
    use malevich::stat::Bins;

    let mut extent: Option<(f64, f64)> = None;
    for &value in values {
        if value.is_finite() {
            let (lo, hi) = extent.get_or_insert((value, value));
            *lo = lo.min(value);
            *hi = hi.max(value);
        }
    }
    let Some((min, max)) = extent else {
        return Ok(None);
    };
    let start = (min / width).floor() * width;
    let count = ((max - start) / width).ceil().max(1.0);
    if !count.is_finite() || count > crate::args::MAX_BINS as f64 {
        return Err(malevich::Error::DimensionTooLarge {
            what: "histogram bin count",
            requested: usize::MAX,
            limit: crate::args::MAX_BINS,
        });
    }
    let mut bins = Bins::try_new(start, width, count as usize)?;
    for &value in values {
        bins.add(value);
    }
    Ok(Some(bins))
}

fn distribution(table: &Table, kind: DistributionKind) -> (Chart, usize) {
    let (values, unparsed) = series::flatten(table);
    (Chart::Distribution { kind, values }, unparsed)
}

fn grouped(table: &Table, kind: GroupedKind) -> (Chart, usize) {
    let (categories, groups, unparsed) = series::groups(table);
    (
        Chart::Grouped {
            kind,
            categories,
            groups,
        },
        unparsed,
    )
}

#[cfg(test)]
#[path = "tests/recipe_tests.rs"]
mod tests;
