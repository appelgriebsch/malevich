//! [`crate::recipe::Recipe`] → [`Plot`]. Zero input interpretation and zero
//! rendering logic: each recipe shape maps directly onto public malevich grammar.

use malevich::scale::Unit;
use malevich::stat::Normalization;
use malevich::{Bars, Cells, Line, Plot, Points, Rule, Scale};

use crate::args::BarLayout;
use crate::recipe::{Chart, DistributionKind, Furniture, GroupedKind, Recipe, ValueMark};
use crate::series::{Dataset, Series};

/// A built plot plus the count of fields that would not parse.
pub struct Built<'a> {
    pub plot: Plot<'a>,
    pub unparsed: usize,
}

/// Builds a retained plot by borrowing the normalized buffers in `recipe`.
pub fn build(recipe: &Recipe) -> malevich::Result<Built<'_>> {
    let plot = match &recipe.chart {
        Chart::Value { mark, data } => value_plot(data, *mark),
        Chart::ScatterBy { x, y, groups } => {
            Plot::new().layer(Points::xy(x, y).color_by(groups.iter().map(String::as_str)))
        }
        Chart::Histogram {
            start,
            width,
            heights,
            normalization,
        } => {
            let plot = Plot::new().layer(Bars::spans(*start, *width, heights));
            // The axis the preset gives each normalization: whole counts,
            // a percent sign, or plain.
            match normalization {
                Normalization::Count => plot.y_scale(Scale::Integer),
                Normalization::Percent => plot.y_unit(Unit::suffix("%")),
                _ => plot,
            }
        }
        Chart::Bars {
            labels,
            values,
            horizontal,
        } => {
            let plot = if *horizontal {
                Plot::new().layer(Bars::new(labels.iter().map(String::as_str), values).horizontal())
            } else {
                malevich::bar(labels.iter().map(String::as_str), values)
            };
            // Frequencies are whole: the count chart's axis says so.
            if recipe.command == crate::args::Command::Count {
                plot.y_scale(Scale::Integer)
            } else {
                plot
            }
        }
        Chart::BarGroups {
            labels,
            names,
            series,
            layout,
            horizontal,
        } => bar_groups(labels, names, series, *layout, *horizontal),
        Chart::Describe { names, groups } => malevich::describe(
            names.iter().map(String::as_str),
            groups.iter().map(Vec::as_slice),
        ),
        Chart::Table {
            rows,
            columns,
            values,
        } => malevich::try_table(
            rows.iter().map(String::as_str),
            columns.iter().map(String::as_str),
            values,
        )?,
        Chart::Distribution { kind, values } => match kind {
            DistributionKind::Density => malevich::density(values),
            DistributionKind::Ecdf => malevich::ecdf(values),
        },
        Chart::Spark { values } => malevich::sparkline(values),
        Chart::Grouped {
            kind,
            categories,
            groups,
        } => match kind {
            GroupedKind::Box => malevich::box_plot(
                categories.iter().map(String::as_str),
                groups.iter().map(Vec::as_slice),
            ),
            GroupedKind::Violin => malevich::violin_with(
                categories.iter().map(String::as_str),
                groups.iter().map(Vec::as_slice),
                malevich::ViolinOptions::default(),
            )?,
        },
        Chart::Grid {
            columns,
            values,
            extents,
            colormap,
            labels_x,
            labels_y,
            reduce,
        } => {
            let mut cells = Cells::try_matrix(*columns, values)?.colormap(colormap.clone());
            if let Some(reducer) = reduce {
                cells = cells.reduce(*reducer);
            }
            if let Some((x, y)) = extents {
                cells = cells.try_extents(*x, *y)?;
            }
            let mut plot = Plot::new().layer(cells).colorbar();
            if let Some(labels) = labels_x {
                plot = plot.x_scale(Scale::bands(labels.iter().map(String::as_str)));
            }
            if let Some(labels) = labels_y {
                plot = plot.y_scale(Scale::bands(labels.iter().map(String::as_str)));
            }
            plot
        }
        Chart::Empty => Plot::new(),
    };
    Ok(Built {
        plot: recipe.furniture.apply(plot),
        unparsed: recipe.unparsed,
    })
}

/// Several value columns per label as stacked or grouped bars — the
/// library's `stack` and `dodge` stats feeding `Bars::base` and `Bars::at`,
/// exactly as the gallery composes them.
fn bar_groups<'a>(
    labels: &'a [String],
    names: &'a [String],
    series: &'a [Vec<f64>],
    layout: BarLayout,
    horizontal: bool,
) -> Plot<'a> {
    let slices: Vec<&[f64]> = series.iter().map(Vec::as_slice).collect();
    let sideways = |bars: Bars<'a>| if horizontal { bars.horizontal() } else { bars };
    match layout {
        BarLayout::Stack | BarLayout::Single => {
            let bands = malevich::stat::stack(&slices);
            bands.into_iter().zip(series).zip(names).fold(
                Plot::new(),
                |plot, (((low, _), values), name)| {
                    let bars = Bars::new(labels.iter().map(String::as_str), values.as_slice())
                        .base(low)
                        .label(name.as_str());
                    plot.layer(sideways(bars))
                },
            )
        }
        BarLayout::Group => {
            let step = 0.8 / slices.len().max(1) as f64;
            let positions = malevich::stat::dodge(&slices, step);
            let plot = positions.into_iter().zip(series).zip(names).fold(
                Plot::new(),
                |plot, ((at, values), name)| {
                    let bars = Bars::at(at, step * 0.9, values.as_slice()).label(name.as_str());
                    plot.layer(sideways(bars))
                },
            );
            let bands = Scale::bands(labels.iter().map(String::as_str));
            if horizontal {
                plot.y_scale(bands)
            } else {
                plot.x_scale(bands)
            }
        }
    }
}

/// Line and scatter: one layer per normalized series.
fn value_plot<'a>(data: &'a Dataset, mark: ValueMark) -> Plot<'a> {
    data.series
        .iter()
        .fold(Plot::new(), |plot, series| layer(plot, data, series, mark))
}

fn layer<'a>(plot: Plot<'a>, data: &'a Dataset, series: &Series, mark: ValueMark) -> Plot<'a> {
    let label = series.label.as_deref();
    let y = data.y(series);
    match (mark, data.x(series)) {
        (ValueMark::Line, Some(x)) => {
            plot.layer(named(Line::xy(x, y), label, |mark, text| mark.label(text)))
        }
        (ValueMark::Line, None) => {
            plot.layer(named(Line::y(y), label, |mark, text| mark.label(text)))
        }
        (ValueMark::Scatter, Some(x)) => {
            plot.layer(named(Points::xy(x, y), label, |mark, text| {
                mark.label(text)
            }))
        }
        (ValueMark::Scatter, None) => {
            plot.layer(named(Points::y(y), label, |mark, text| mark.label(text)))
        }
    }
}

fn named<M>(mark: M, label: Option<&str>, set: impl FnOnce(M, &str) -> M) -> M {
    match label {
        Some(text) => set(mark, text),
        None => mark,
    }
}

impl Furniture {
    /// Applies the shared title, axes, domains, and scale choices.
    pub(crate) fn apply<'a>(&self, mut plot: Plot<'a>) -> Plot<'a> {
        for &value in &self.hlines {
            plot = plot.layer(Rule::h(value));
        }
        for &value in &self.vlines {
            plot = plot.layer(Rule::v(value));
        }
        if let Some(unit) = &self.unit {
            plot = if self.unit_on_x {
                plot.x_unit(unit.clone())
            } else {
                plot.y_unit(unit.clone())
            };
        }
        if let Some(title) = &self.title {
            plot = plot.title(title);
        }
        if let Some(xlabel) = &self.xlabel {
            plot = plot.x_label(xlabel);
        }
        if let Some(ylabel) = &self.ylabel {
            plot = plot.y_label(ylabel);
        }
        if let Some((lo, hi)) = self.xlim {
            plot = plot.x_domain(lo, hi);
        }
        if let Some((lo, hi)) = self.ylim {
            plot = plot.y_domain(lo, hi);
        }
        if self.time_x {
            plot = plot.time_x();
        }
        if self.log_x {
            plot = plot.log_x();
        }
        if self.log_y {
            plot = plot.log_y();
        }
        plot
    }
}
