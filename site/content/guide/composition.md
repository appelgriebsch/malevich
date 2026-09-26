# Composition

Composition over modes. A `Grid` pastes independently rendered plots side by side; shared axes are an explicit composition — fix the domains — never a hidden linking mode; a table sits beside a chart because both are plots that render to strings. This page collects the compositions people ask for.

## Small multiples

`Grid::new(columns)` takes plots with `with` and fills rows left to right. Each pane resolves its own domains; padding is escape-aware, so colored panes align.

{{example multiples}}

To share an axis, fix it: `x_domain` on every pane, or one `Viewport` applied to all of them in an interactive host. There is no `sharex` flag because the composition already says everything the flag would.

## Two series, two scales

Twin y axes are refused: a second scale in one panel lets two series lie about their relative magnitude. The honest form is two panels that share the x window, so the eye compares shapes without a fabricated common magnitude.

{{pair comp_shared_a comp_shared_b}}

```rust
use malevich::{Frame, Grid, Line, Plot};

let window = (0.0, 199.0);
let grid = Grid::new(1)
    .with(Plot::new().layer(Line::xy(&x[..], &rate[..])).x_domain(window.0, window.1).title("rate"))
    .with(Plot::new().layer(Line::xy(&x[..], &count[..])).x_domain(window.0, window.1).title("count"));
println!("{}", grid.render(&Frame::plain(60, 20)));
```

## A chart with its numbers

`describe` and `table` are plots — text on band axes — so a summary table renders beside its chart with no figure API at all: two renders, printed in order, at the same width. The box plot's own quartiles reappear in the p25, p50, and p75 columns because both use the one type-7 estimator.

{{example firstlook}}

`table_with` colors each value through a colormap positioned within its own column — the heatmap reading of a matrix, with the digits still carrying the value in any pipe.

{{figure comp_table}}

{{example seasons}}

## The pie's honest forms

A pie encodes parts by angle, which the eye ranks poorly and a cell grid cannot draw. Two compositions read the parts against a straight axis.

The waffle: one hundred cells, each a category, as `Cells::classes` over a ten-by-ten grid with the axes off. Shares round to whole cells the eye can count; the legend names them.

{{example waffle}}

The breakdown: bars from a `stack`, with `StackOffset::Normalize` when the parts should sum to 100 %. Every region's sources as a horizontal stack, and a region with nothing to show draws nothing.

{{example breakdown}}

## Tornado and waterfall

A tornado is horizontal bars around zero, the longest first — sort by absolute swing and the shape appears. A waterfall is `Bars::base` with each bar's base at the previous running total.

{{figure comp_tornado}}

## Stacked and grouped bars

Never a preset: `Bars::base` stacks (the low half of `stat::stack`), and `Bars::at` at positions from `stat::dodge` groups. Vertical or sideways, the same two lines.

{{example segments}}

{{example speedup}}

## Annotating a matrix

A confusion matrix with its counts, a correlation matrix with its coefficients: `Cells::matrix` on two band axes, plus one `Text` per cell with `Align::Center`. The annotation keeps the cell's color as its background and picks its ink from the luminance underneath; in plain output the digits stand beside the shades.

{{example confusion}}

## Ridgelines and rainclouds

Rows rendered back to front at fixed elevation — a lifted KDE per row in the corners style, so nearer rows overwrite what they cross — are the terminal's honest 3D surface. A raincloud is a half-violin cloud, a `Range` box, and every measurement as jittered rain.

{{example ridgeline}}

## What composition will not be

There is no figure object, no subplot grid with shared-axis modes, no layout manager. A plot renders to a `String`; strings concatenate; `Grid` does the one thing concatenation gets wrong (padding colored rows to equal width). Everything else a figure API would offer is either a domain you can fix or a mark you can add.
