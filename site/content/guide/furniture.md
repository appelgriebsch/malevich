# Furniture

Furniture is everything on a chart that is not data: the title, the axis labels, the legend, the colorbar, the ticks and their labels, and the axes themselves. Malevich lays it out with one rule — furniture sheds before data — and one switch, `axes(false)`, for the charts that want none.

## The pieces

{{figure furniture_all}}

| piece | comes from | shed order |
|---|---|---|
| title | `Plot::title` | second |
| axis labels | `x_label`, `y_label` | with the title row |
| legend | any layer with a `label`, or a `color_by` channel | first |
| colorbar | `Plot::colorbar` on a plot with a colormap | with the legend |
| context note | automatic, when labels leave out a base or a date | y note with the legend, x note with the title |
| tick labels | computed | last, by thinning |

The legend is built from layer labels in layer order; `color_by` adds one entry per category in first-appearance order, and `Cells::classes` adds swatches. A colorbar labels its ramp with the same exact-decimal formatter as the axes — decade ticks on a log colormap, band boundaries on a stepped one.

## Shedding

When the frame shrinks, the layout sheds furniture instead of failing: legend, then titles, then tick density. The data region is the last thing standing, because a small chart of the real numbers beats a complete frame around nothing. `TERM=dumb` at any width still gets a correct chart.

{{sizes hero 80x18 56x14 40x10 26x7 16x4 | One plot value, five frames. Watch the legend go first, then the title and axis labels, then the tick density, and note that the y labels keep their exact decimals to the end.}}

Rendering therefore never fails on frame grounds; panics belong to construction, at the caller's line, on documented programmer invariants (unequal paired channels, a zero-column grid). A spec that arrives from data — deserialization, a config file — gets the checked twins, `Plot::validate` and `Plot::try_render`, which report the first problem as a typed error instead.

## No axes at all

`axes(false)` removes the axes, tick labels, and gutters, so the data region fills the frame. The `sparkline` preset is bars from zero with the axes off in a one-row frame; a waffle is class cells with the axes off; a thumbnail in a dashboard is anything with the axes off.

{{figure furniture_sparkline}}

{{figure furniture_axes_off}}

## The card

The HTML and SVG cards add one more piece of furniture that a tty does not have: the card itself — a rounded rectangle in the theme's background and foreground. `Theme::LIGHT` selects the light card; every other theme takes the dark one. The grid inside is the exact grid the terminal renderer would print.

{{light hero}}

## What furniture will not do

There is no legend placement option, no title alignment option, no gutter width option, no font. Each would be a knob on presentation that the frame already decides, and each would be one more thing the layout could not shed. Text the chart needs in a particular place is a `Text` mark at data coordinates; the rest is computed.
