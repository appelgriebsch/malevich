//! Presets: chart types as plain functions over the grammar.
//!
//! Every preset is a composition of marks, scales, and furniture — nothing a preset
//! does is beyond reach of the grammar, and each returns the [`Plot`] for refinement.

use crate::data::IntoSeries;
use crate::mark::{Area, Bars, Cells, Line, Points, Range};
use crate::plot::Plot;

/// A line chart of `values` plotted against their indices.
///
/// ```
/// let chart = malevich::line(&[1.0, 4.0, 2.0, 8.0][..]);
/// println!("{}", chart.render(&malevich::Frame::plain(40, 10)));
/// ```
pub fn line<'a>(values: impl IntoSeries<'a>) -> Plot<'a> {
    Plot::new().layer(Line::y(values))
}

/// A scatter chart of the points `(x[i], y[i])`.
///
/// ```
/// let chart = malevich::scatter(&[1.0, 2.0, 3.0][..], &[2.0, 1.0, 3.0][..]);
/// println!("{}", chart.render(&malevich::Frame::plain(40, 10)));
/// ```
pub fn scatter<'a>(x: impl IntoSeries<'a>, y: impl IntoSeries<'a>) -> Plot<'a> {
    Plot::new().layer(Points::xy(x, y))
}

/// A bar chart: one labeled bar per category, rising from zero.
///
/// ```
/// let chart = malevich::bar(["a", "b", "c"], &[3.0, 7.0, 5.0][..]);
/// println!("{}", chart.render(&malevich::Frame::plain(40, 10)));
/// ```
pub fn bar<'a>(
    categories: impl IntoIterator<Item = impl Into<String>>,
    values: impl IntoSeries<'a>,
) -> Plot<'a> {
    Plot::new().layer(Bars::new(categories, values))
}

/// A histogram: `values` binned automatically (Sturges/Freedman–Diaconis, nice
/// decimal edges) and drawn as contiguous bars from zero.
///
/// ```
/// let samples = [1.0, 2.0, 2.5, 2.7, 3.0, 3.1, 3.2, 4.0, 5.5];
/// println!("{}", malevich::hist(&samples[..]).render(&malevich::Frame::plain(40, 10)));
/// ```
pub fn hist<'a>(values: impl IntoSeries<'a>) -> Plot<'a> {
    let series = values.into_series();
    match crate::stat::Bins::auto(series.as_slice(), 60) {
        Some(bins) => {
            let counts: Vec<f64> = bins.counts().iter().map(|&count| count as f64).collect();
            Plot::new().layer(Bars::spans(bins.start(), bins.width(), counts))
        }
        None => Plot::new(),
    }
}

/// A step chart: `values` held flat between indices — counters, rates, states.
///
/// ```
/// println!("{}", malevich::stairs(&[1.0, 3.0, 2.0][..]).render(&malevich::Frame::plain(40, 8)));
/// ```
pub fn stairs<'a>(values: impl IntoSeries<'a>) -> Plot<'a> {
    let series = values.into_series();
    let mut x = Vec::with_capacity(series.len() * 2);
    let mut y = Vec::with_capacity(series.len() * 2);
    for (index, value) in series.iter().enumerate() {
        if index > 0 {
            x.push(index as f64);
            y.push(y.last().copied().unwrap_or(value));
        }
        x.push(index as f64);
        y.push(value);
    }
    Plot::new().layer(Line::xy(x, y))
}

/// An ECDF chart: the fraction of `values` at or below each value, as a step line
/// from 0 to 1.
///
/// ```
/// let samples = [3.0, 1.0, 4.0, 1.0, 5.0];
/// println!("{}", malevich::ecdf(&samples[..]).render(&malevich::Frame::plain(40, 8)));
/// ```
pub fn ecdf<'a>(values: impl IntoSeries<'a>) -> Plot<'a> {
    let series = values.into_series();
    let (sorted, fractions) = crate::stat::ecdf(series.as_slice());
    let mut x = Vec::with_capacity(sorted.len() * 2);
    let mut y = Vec::with_capacity(sorted.len() * 2);
    let mut previous = 0.0f64;
    for (value, fraction) in sorted.into_iter().zip(fractions) {
        x.push(value);
        y.push(previous);
        x.push(value);
        y.push(fraction);
        previous = fraction;
    }
    Plot::new().layer(Line::xy(x, y))
}

/// A heatmap of a row-major grid, `columns` wide: shade-ramp glyphs colored by the
/// default colormap. Row 0 is the bottom row.
///
/// ```
/// let grid = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
/// println!("{}", malevich::heatmap(3, &grid[..]).render(&malevich::Frame::plain(30, 8)));
/// ```
pub fn heatmap<'a>(columns: usize, values: impl IntoSeries<'a>) -> Plot<'a> {
    Plot::new().layer(Cells::matrix(columns, values))
}

/// A 2D histogram: point density on a uniform grid over the data's extent.
///
/// ```
/// let x = [1.0, 1.1, 5.0, 5.1, 5.2];
/// let y = [2.0, 2.1, 8.0, 8.1, 7.9];
/// println!("{}", malevich::hist2d(&x[..], &y[..]).render(&malevich::Frame::plain(40, 12)));
/// ```
pub fn hist2d<'a>(x: impl IntoSeries<'a>, y: impl IntoSeries<'a>) -> Plot<'a> {
    let xs = x.into_series();
    let ys = y.into_series();
    match crate::stat::bins2(xs.as_slice(), ys.as_slice(), 48, 32) {
        Some(grid) => {
            // Empty bins are gaps, not the faintest shade — blank space must mean
            // "no data", never "a little data".
            let counts: Vec<f64> = grid
                .counts
                .into_iter()
                .map(|count| if count == 0.0 { f64::NAN } else { count })
                .collect();
            Plot::new().layer(Cells::matrix(grid.columns, counts).extents(grid.x, grid.y))
        }
        None => Plot::new(),
    }
}

/// Box plots: one five-number box per category (type-7 quartiles, Tukey whiskers),
/// with outliers as dots.
///
/// ```
/// let a = [1.0, 2.0, 3.0, 4.0, 9.0];
/// let b = [2.0, 4.0, 5.0, 6.0, 7.0];
/// let chart = malevich::box_plot(["a", "b"], [&a[..], &b[..]]);
/// println!("{}", chart.render(&malevich::Frame::plain(40, 12)));
/// ```
///
/// # Panics
///
/// Panics if the number of categories differs from the number of groups.
pub fn box_plot<'a>(
    categories: impl IntoIterator<Item = impl Into<String>>,
    groups: impl IntoIterator<Item = impl IntoSeries<'a>>,
) -> Plot<'a> {
    let categories: Vec<String> = categories.into_iter().map(Into::into).collect();
    let stats: Vec<Option<crate::stat::BoxStats>> = groups
        .into_iter()
        .map(|group| crate::stat::BoxStats::of(group.into_series().as_slice()))
        .collect();
    assert_eq!(
        categories.len(),
        stats.len(),
        "box_plot requires one category per group"
    );
    let pick = |f: &dyn Fn(&crate::stat::BoxStats) -> f64| -> Vec<f64> {
        stats
            .iter()
            .map(|s| s.as_ref().map_or(f64::NAN, f))
            .collect()
    };
    let mut outlier_x = Vec::new();
    let mut outlier_y = Vec::new();
    for (index, stat) in stats.iter().enumerate() {
        if let Some(stat) = stat {
            for &outlier in &stat.outliers {
                outlier_x.push(index as f64);
                outlier_y.push(outlier);
            }
        }
    }
    let plot = Plot::new().layer(
        Range::over(
            categories,
            pick(&|s| s.whisker_low),
            pick(&|s| s.whisker_high),
        )
        .body(pick(&|s| s.q1), pick(&|s| s.q3))
        .marker(pick(&|s| s.median)),
    );
    if outlier_x.is_empty() {
        plot
    } else {
        plot.layer(Points::xy(outlier_x, outlier_y))
    }
}

/// Error bars: points with symmetric `error` intervals around each `y`.
///
/// ```
/// let x = [1.0, 2.0, 3.0];
/// let y = [4.0, 6.0, 5.0];
/// let e = [0.5, 1.0, 0.4];
/// println!("{}", malevich::error_bars(&x[..], &y[..], &e[..]).render(&malevich::Frame::plain(40, 10)));
/// ```
///
/// # Panics
///
/// Panics if the series have different lengths.
pub fn error_bars<'a>(
    x: impl IntoSeries<'a>,
    y: impl IntoSeries<'a>,
    error: impl IntoSeries<'a>,
) -> Plot<'a> {
    let x = x.into_series();
    let y = y.into_series();
    let error = error.into_series();
    assert!(
        x.len() == y.len() && y.len() == error.len(),
        "error_bars requires series of equal length"
    );
    let low: Vec<f64> = y.iter().zip(error.iter()).map(|(y, e)| y - e).collect();
    let high: Vec<f64> = y.iter().zip(error.iter()).map(|(y, e)| y + e).collect();
    let xs = x.as_slice().to_vec();
    Plot::new()
        .layer(Range::xy(xs.clone(), low, high))
        .layer(Points::xy(xs, y.as_slice().to_vec()))
}

/// A density chart: the Gaussian KDE of `values` as a smooth line.
///
/// ```
/// let samples = [1.0, 2.0, 2.5, 2.7, 3.0, 3.2, 4.0];
/// println!("{}", malevich::density(&samples[..]).render(&malevich::Frame::plain(40, 10)));
/// ```
pub fn density<'a>(values: impl IntoSeries<'a>) -> Plot<'a> {
    let series = values.into_series();
    match crate::stat::kde(series.as_slice(), 256) {
        Some((positions, densities)) => Plot::new().layer(Line::xy(positions, densities)),
        None => Plot::new(),
    }
}

/// Violin plots: one mirrored density per category, each scaled to the same width.
///
/// ```
/// let a = [1.0, 2.0, 2.5, 3.0, 3.5];
/// let b = [4.0, 5.0, 5.5, 6.0, 8.0];
/// let chart = malevich::violin(["a", "b"], [&a[..], &b[..]]);
/// println!("{}", chart.render(&malevich::Frame::plain(44, 12)));
/// ```
///
/// # Panics
///
/// Panics if the number of categories differs from the number of groups.
pub fn violin<'a>(
    categories: impl IntoIterator<Item = impl Into<String>>,
    groups: impl IntoIterator<Item = impl IntoSeries<'a>>,
) -> Plot<'a> {
    let categories: Vec<String> = categories.into_iter().map(Into::into).collect();
    let densities: Vec<Option<(Vec<f64>, Vec<f64>)>> = groups
        .into_iter()
        .map(|group| crate::stat::kde(group.into_series().as_slice(), 128))
        .collect();
    assert_eq!(
        categories.len(),
        densities.len(),
        "violin requires one category per group"
    );
    let count = categories.len();
    // A data-free band-placed range declares the categorical axis; the violins
    // themselves are horizontal areas over the band centers.
    let gaps = vec![f64::NAN; count];
    let mut plot = Plot::new().layer(Range::over(categories, gaps.clone(), gaps));
    for (index, density) in densities.into_iter().enumerate() {
        let Some((positions, values)) = density else {
            continue;
        };
        let peak = values.iter().copied().fold(f64::MIN_POSITIVE, f64::max);
        let center = index as f64;
        let half: Vec<f64> = values.iter().map(|v| v / peak * 0.35).collect();
        let left: Vec<f64> = half.iter().map(|w| center - w).collect();
        let right: Vec<f64> = half.iter().map(|w| center + w).collect();
        plot = plot.layer(Area::horizontal(positions, left, right));
    }
    plot
}
