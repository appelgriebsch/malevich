//! The view layer: pure functions from catalog data to `malevich::Plot`s.
//!
//! Nothing here reads state or owns a terminal — every function takes data and
//! options in and returns owned plots, so views render identically in the TUI, in
//! the headless `--render` mode, and under test.

use malevich::{Area, Cells, Color, Line, LineStyle, Plot, Points, Rule};

use super::data::{Catalog, Kind, Series, align, extent, step_series};

/// The app's screens, in tab order.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Every series at a glance — small multiples.
    Overview,
    /// One series large, with transforms and recession shading.
    Series,
    /// How the series' changes distribute: histogram plus decade box plots.
    Distribution,
    /// Period changes as a year × period heatmap.
    Seasonality,
    /// Cross-series classics: the Phillips curve and the yield spread.
    Relations,
}

impl View {
    pub const ALL: [View; 5] = [
        View::Overview,
        View::Series,
        View::Distribution,
        View::Seasonality,
        View::Relations,
    ];

    pub fn title(self) -> &'static str {
        match self {
            View::Overview => "overview",
            View::Series => "series",
            View::Distribution => "distribution",
            View::Seasonality => "seasonality",
            View::Relations => "relations",
        }
    }

    pub fn next(self) -> View {
        let index = Self::ALL.iter().position(|v| *v == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    pub fn previous(self) -> View {
        let index = Self::ALL.iter().position(|v| *v == self).unwrap_or(0);
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

/// What the series view draws: the level as reported, its change over one year, or
/// the level on a logarithmic axis.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Transform {
    Level,
    YearOverYear,
    Log,
}

impl Transform {
    pub fn next(self) -> Transform {
        match self {
            Transform::Level => Transform::YearOverYear,
            Transform::YearOverYear => Transform::Log,
            Transform::Log => Transform::Level,
        }
    }

    pub fn label(self, kind: Kind) -> &'static str {
        match (self, kind) {
            (Transform::Level, _) => "level",
            (Transform::YearOverYear, Kind::Index) => "year-over-year %",
            (Transform::YearOverYear, Kind::Rate) => "1-year change, points",
            (Transform::Log, _) => "level (log axis)",
        }
    }
}

/// One small-multiple line chart per series, for the overview grid.
pub fn overview_charts(catalog: &Catalog) -> Vec<Plot<'static>> {
    catalog
        .series
        .iter()
        .map(|series| {
            let latest = series
                .latest()
                .map(|v| format!("{v:.1}"))
                .unwrap_or_default();
            Plot::new()
                .layer(Line::xy(series.dates.clone(), series.values.clone()).color(Color::Cyan))
                .title(format!("{}  {latest}", series.id))
                .time_x()
        })
        .collect()
}

/// The main chart: one series under a transform, optionally with the NBER
/// recessions marked, in the chosen line style. The fed funds rate draws as steps —
/// an administered rate holds until the next decision, and the chart should say so.
pub fn series_chart(
    series: &Series,
    transform: Transform,
    style: LineStyle,
    recessions: Option<&[(f64, f64)]>,
) -> Plot<'static> {
    let (x, y) = match transform {
        Transform::Level | Transform::Log => (series.dates.clone(), series.values.clone()),
        Transform::YearOverYear => (series.dates.clone(), series.year_over_year()),
    };
    let stepped = series.id == "FEDFUNDS" && transform != Transform::YearOverYear;
    let (x, y) = if stepped { step_series(&x, &y) } else { (x, y) };

    let mut plot = Plot::new()
        .title(format!("{} ({})", series.title, series.id))
        .time_x()
        .y_label(match transform {
            Transform::YearOverYear => match series.kind {
                Kind::Index => "percent",
                Kind::Rate => "points",
            },
            _ => series.unit,
        });

    // NBER recessions as a ribbon in a strip reserved *below* the data — a
    // full-height band would fill every subpixel and swallow the line (terminals
    // have no translucency). Linear axes only; a log axis must stay positive.
    let (lo, hi) = extent(&y);
    if let Some(recessions) = recessions
        && transform != Transform::Log
        && lo.is_finite()
        && hi > lo
    {
        let strip = (hi - lo) * 0.06;
        plot = plot.y_domain(lo - strip, hi);
        let (first, last) = (x[0], *x.last().unwrap_or(&0.0));
        for &(start, end) in recessions {
            if end >= first && start <= last {
                plot = plot.layer(
                    Area::between([start, end], [lo - strip, lo - strip], [lo, lo])
                        .color(Color::Red),
                );
            }
        }
    }

    // Inflation charts get the Fed's 2% target as a reference rule.
    if series.id == "CPIAUCSL" && transform == Transform::YearOverYear {
        plot = plot.layer(Rule::h(2.0).label("2% target").color(Color::Yellow));
    }

    plot = plot.layer(Line::xy(x, y).style(style).color(Color::Cyan));
    if transform == Transform::Log {
        plot = plot.log_y();
    }
    plot
}

/// The distribution view: how the series' period changes distribute, and how its
/// level moved by decade. Returns `(histogram, decade box plots)`.
pub fn distribution_charts(series: &Series) -> (Plot<'static>, Plot<'static>) {
    let changes = series.period_changes();
    let change_unit = match series.kind {
        Kind::Index => "% change per period",
        Kind::Rate => "points change per period",
    };
    let histogram = malevich::hist(changes)
        .title(format!("{}: distribution of changes", series.id))
        .x_label(change_unit);

    let (decades, groups) = series.by_decade();
    let boxes = malevich::box_plot(decades, groups)
        .title(format!("{}: level by decade", series.id))
        .y_label(series.unit);
    (histogram, boxes)
}

/// The seasonality view: period changes as a year × period heatmap (rows are
/// years, oldest at the bottom; columns run January → December), with a colorbar.
pub fn seasonality_chart(series: &Series, rows: usize) -> Plot<'static> {
    let (columns, grid, first_year, last_year) = series.seasonality(rows);
    let period = if columns == 4 {
        "quarter"
    } else {
        "month (Jan → Dec)"
    };
    let change = match series.kind {
        Kind::Index => "% change",
        Kind::Rate => "points change",
    };
    if grid.is_empty() {
        return Plot::new().title(format!("{}: no data", series.id));
    }
    Plot::new()
        .layer(Cells::matrix(columns, grid).extents(
            (0.0, columns as f64),
            (first_year as f64, last_year as f64 + 1.0),
        ))
        .colorbar()
        .title(format!("{}: {change} by {period} and year", series.id))
        .x_label(period)
}

/// The relations view: the Phillips curve (unemployment vs inflation, split at
/// 2000 to show the flattening) and the 10y − fed-funds yield spread whose
/// inversions precede recessions. Returns `(phillips, spread)`.
pub fn relations_charts(catalog: &Catalog) -> (Plot<'static>, Plot<'static>) {
    let unrate = catalog.by_id("UNRATE").expect("vendored");
    let cpi = catalog.by_id("CPIAUCSL").expect("vendored");
    let gs10 = catalog.by_id("GS10").expect("vendored");
    let fedfunds = catalog.by_id("FEDFUNDS").expect("vendored");

    // Unemployment against CPI inflation on the same month, split into eras.
    let inflation = cpi.year_over_year();
    let cutoff = super::data::parse_date("2000-01-01").expect("valid date");
    let mut phillips = Plot::new()
        .title("Phillips curve: unemployment vs inflation, monthly")
        .x_label("unemployment %")
        .y_label("CPI YoY %");
    let (dates, unemployment, inflation) =
        align(&unrate.dates, &unrate.values, &cpi.dates, &inflation);
    for (label, keep) in [
        ("1948-1999", true), // keep dates before the cutoff
        ("2000-now", false), // keep dates at or after it
    ] {
        let (u, i): (Vec<f64>, Vec<f64>) = dates
            .iter()
            .zip(unemployment.iter().zip(&inflation))
            .filter(|(date, _)| (**date < cutoff) == keep)
            .map(|(_, (&u, &i))| (u, i))
            .unzip();
        phillips = phillips.layer(Points::xy(u, i).label(label));
    }

    // The spread between the 10-year yield and the policy rate, with inversion at 0.
    let (dates, ten_year, funds) =
        align(&gs10.dates, &gs10.values, &fedfunds.dates, &fedfunds.values);
    let spread: Vec<f64> = ten_year.iter().zip(&funds).map(|(a, b)| a - b).collect();
    let spread_plot = Plot::new()
        .layer(Rule::h(0.0).label("inversion").color(Color::Red))
        .layer(
            Line::xy(dates, spread)
                .label("10y - fed funds")
                .color(Color::Cyan),
        )
        .title("Yield spread: 10-year minus fed funds")
        .time_x()
        .y_label("points");
    (phillips, spread_plot)
}
