# Recipes

These answer the requests that show up most, each with what already exists.
Every chart type below is a composition of the grammar. None needed a new
concept ([what earns one](principles/what-earns-a-concept.md)).

## Benchmarks through `jq`

[hyperfine](https://github.com/sharkdp/hyperfine) exports one JSON file per
run. Its `results` carry a mean and every timing. Means as sideways bars in
seconds, then one command's distribution:

```sh
hyperfine --export-json bench.json 'sort big.txt' 'sort --parallel=4 big.txt'
jq -r '.results[] | [.command, .mean] | @tsv' bench.json | kaz bar --horizontal --unit s
jq -r '.results[0].times[]' bench.json | kaz hist --unit s
```

[Criterion](https://github.com/bheisler/criterion.rs) writes
`target/criterion/<bench>/new/estimates.json` (nanoseconds) and the samples
beside it:

```sh
for run in target/criterion/*/new; do
  printf '%s %s\n' "$(basename "$(dirname "$run")")" "$(jq .mean.point_estimate "$run/estimates.json")"
done | kaz bar --horizontal --unit ns
jq -r '.times[]' target/criterion/parse/new/sample.json | kaz density --unit ns
```

`kaz describe` prints the same numbers as a table. `--emit-code` on any of
them writes the Rust program.

## The pie: a waffle or a breakdown

A pie encodes parts by angle, which the eye ranks poorly and a cell grid
cannot draw. Two compositions read the parts against a straight axis:

- **The waffle**: one hundred cells, each a category, as `Cells::classes`
  over a ten-by-ten grid with the axes off — the `waffle` gallery example.
  Shares round to whole cells and the legend names them.
- **The breakdown**: `Bars` from a `stack` — the `breakdown` example — with
  `StackOffset::Normalize` when the parts should sum to 100 %.

## Tornado and breakdown bars

A tornado is horizontal bars around zero, the longest first:

```rust
use malevich::{Bars, Frame, Plot, Rule};

let factors = ["price", "volume", "fx", "cost"];
let swing = [-4.0, 3.5, -1.2, 0.8];
let plot = Plot::new()
    .layer(Bars::new(factors, &swing[..]).horizontal())
    .layer(Rule::v(0.0));
println!("{}", plot.render(&Frame::plain(60, 10)));
```

Sort the categories by `|swing|` first and the shape appears. A breakdown
stacks segments with `Bars::base`, one layer per part.

## Two series, two scales: a `Grid` and a shared window

Twin y axes are refused ([why](../README.md#what-it-will-not-be)). The
honest form is two panels that share the x window, so the eye compares
shapes without a fabricated common magnitude:

```rust
use malevich::{Frame, Grid, Line, Plot};

let x: Vec<f64> = (0..200).map(f64::from).collect();
let rate: Vec<f64> = x.iter().map(|v| 3.0 + (v * 0.05).sin()).collect();
let count: Vec<f64> = x.iter().map(|v| 1000.0 + v * 12.0).collect();
let window = (0.0, 199.0);
let grid = Grid::new(1)
    .with(Plot::new().layer(Line::xy(&x[..], &rate[..])).x_domain(window.0, window.1).title("rate"))
    .with(Plot::new().layer(Line::xy(&x[..], &count[..])).x_domain(window.0, window.1).title("count"));
println!("{}", grid.render(&Frame::plain(60, 20)));
```

Interactively, one `Viewport` applied to both plots keeps them linked
([interaction.md](interaction.md)).

## Positions clip, colors squish

Two out-of-range rules, deliberately different:

- A value outside a fixed axis domain (`x_domain`, `y_min`, a `Viewport`)
  is **clipped**: drawn nowhere, never moved onto the border. A point at
  the edge is a point at the edge, not a refugee from beyond it.
- A value outside a fixed colormap domain (`Colormap::domain`) is
  **squished** into the ramp's nearest end, because a cell must be some
  color. `under` and `over` disclose the squish with colors of their own,
  and the colorbar shows the fixed range.

Gaps follow the first rule everywhere. `NaN` is a break, never interpolated.
A non-positive value on a log axis is a gap.

## Plain text is agent-legible

`Frame::plain` renders the chart as the text it also is: an axis with exact
labels, marks in rows, no escape byte. That is the form a log, a diff, a
test assertion, or a language model reads — and every colored or pixel
rendering degrades to it, so nothing a chart says lives only in color.
