//! Presets: chart types as plain functions over the grammar.
//!
//! Every preset is a composition of marks, scales, and furniture — nothing a preset
//! does is beyond reach of the grammar, and each returns the [`Plot`] for refinement.

use crate::data::IntoSeries;
use crate::mark::{Bars, Line, Points};
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
