# Terminology

The codebase contract: every public concept is named here before it is named in code.
When a concept is added, renamed, or changes meaning, this file is updated in the same
change. Each entry: what the word means in the wider literature, why this word, and what
it maps to in the crate. Types marked *(planned)* do not exist yet.

## Plot

The retained description of a chart: layers plus scales plus furniture (title, labels,
legend). A plain value — cloneable, inspectable, serializable — with no connection to a
terminal. Rendering is a pure function of a `Plot` and a `Frame`. Maps to `plot::Plot`
(re-exported at the root).

## Layer

One mark bound to data and options, stacked with other layers on shared scales — axis
domains are the union of all layers' data, re-resolved at render time. The layering
concept of every grammar of graphics (Wilkinson 2005; Vega-Lite `layer`; UnicodePlots
`lineplot!`). Maps to `Plot::layer`.

## Mark

A family of geometric primitives that draw data: `Line`, `Points`, `Bars`, `Area`,
`Cells`, `Range`, `Rule`, `Text`. The word follows Observable Plot and Vega-Lite
("mark"), chosen over matplotlib's "artist" (too broad) and "geom" (ggplot jargon).
Chart types are compositions of marks, never peers of them. Maps to the `mark`
module — currently `mark::Line` (points, paired series, or a sampled function),
`mark::Points`, `mark::Bars` (bands or numeric spans, zero-baseline), `mark::Area`
(baseline fills and bands), `mark::Rule` (reference lines), and `mark::Text`
(annotations at data coordinates), `mark::Cells` (value grids as shade ramp plus
colormap), and `mark::Range` (intervals with optional body and marker channels —
error bars, boxes, event ticks), joined under the closed `mark::Mark` enum. The
family is complete.

## Channel

A per-mark visual variable fed from data or set constant: `x`, `y`, `y2`, `color`,
`label`, …. Follows Vega-Lite/Observable Plot "encoding channel". Channels accept
anything series-shaped (see Series). Maps to mark constructor arguments and builder
methods *(planned)*.

## Series

One column of scalar data after ingestion: contiguous `f64`, where `NaN` is a gap (see
Gap). The ingestion boundary is the `IntoSeries` trait — slices, arrays, and vectors of
any primitive numeric type convert exactly once at the rim (borrowed `f64` slices are
zero-copy), iterators arrive via `FromIterator`, and function sampling arrives with the
marks. The core is monomorphic `f64`. Maps to `data::Series` and `data::IntoSeries`.

## Stat

A data transform that runs before scales see the data: `Bin`, `Agg`, `Window`, `Stack`,
`Normalize`, `Density`, `Ecdf`, `BoxStats`, `Downsample`, `Contour`. The word follows
seaborn.objects (`Stat`) and ggplot (`stat_*`). Stats are mergeable: two partial results
combine associatively, which is what makes host-side parallelism and streaming
compositions rather than features. Maps to the `stat` module: `stat::M4` (with
`stat::m4`, auto-inserted for large line layers), `stat::Bins`/`stat::bins2`
(histograms), `stat::Agg` (group-by with the shared reducer vocabulary),
`stat::BoxStats` (type-7 quartiles, Tukey whiskers), `stat::kde` (Silverman
bandwidth, linear binning), `stat::Window` (trailing rolling reduces), `stat::ecdf`,
`stat::stack`, `stat::lttb`, and `stat::Moments`.

## Reducer

A named aggregation shared by every aggregating stat: `count`, `sum`, `mean`, `median`,
`min`, `max`, percentiles. One vocabulary across `Bin`, `Agg`, and `Window` (the
Observable Plot convention). Maps to `Reducer` *(planned)*.

## Scale

A mapping from data domain to raster range with the d3-scale contract: `nice`,
`ticks(n)`, `invert`, and a tick formatter. Position scales: `Linear`, `Log`, `Time`,
`Band`; color scales: sequential, diverging, categorical. Maps to the `scale`
module — currently `scale::Linear` (the affine map, including the raster y-flip),
`scale::Band` (categories across a range, d3 padding model), and `scale::Ticks`
(extended-Wilkinson linear placement, `Ticks::log10` decades, and `Ticks::time`
calendar ticks over unix seconds — UTC, exact Gregorian arithmetic); the axis
specification is `scale::Scale` (`Linear | Log | Time | Bands`) set via
`Plot::x_scale`/`y_scale` (with `log_y()`-style sugar kept), axis titles come from
`Plot::x_label`/`y_label` (x centered below, y vertical along the left edge), and
log/time axes are also
enabled per plot with `Plot::log_x`/`Plot::log_y`/`Plot::time_x`, `scale::Colormap` covers the
sequential color scale, and the richer scale-spec API arrives with the time work.

## Ticks

The axis values a scale chooses to label, placed by the extended Wilkinson algorithm
(Talbot, Lin, Hanrahan, InfoVis 2010) — scored for simplicity, coverage, density, and
legibility. Ticks are computed, never supplied as strings, and carry exact-decimal
labels (integer mantissa times a power of ten): labels parse back to their values, share
one fraction width per axis, and never show float artifacts. Maps to `scale::Ticks`.

## Frame

Where and how to render: width and height in cells, charset, color mode (theme joins
later). Frame is render state, not plot state — the same `Plot` renders into many
frames. `Frame::detect()` is the only place the crate inspects the environment
(terminal size, `NO_COLOR`, whether stdout is a terminal); `Frame::plain()` is the
deterministic form. Maps to `plot::Frame` and `plot::ColorMode`.

## Surface

The subpixel grid that marks draw on during rasterization, before glyphs exist
(raster convention: origin top-left, y down; the data-space flip happens in scales). A
charset codec maps each cell's subpixel pattern to one glyph plus a color; text shares
the grid and wins over pixels. Drawing is infallible: out-of-surface clips, non-finite
draws nothing, the last write owns a shared cell's color. Maps to `render::Surface`.

## Charset

A glyph tier used to encode the surface: `Ascii`, `Blocks`, `Braille`, `Sextants`,
`Octants`, or `Auto` (environment-sniffed, never probed). Glyph tables are data, not
code. Maps to `render::Charset`: `Ascii`, `HalfBlocks`, `Quadrants`, `Sextants`
(Unicode 13), `Octants` (Unicode 16 — braille density with solid ink), and
`Braille`. `Frame::detect` sniffs the environment (never probes): octants on
terminals known to render them (kitty, ghostty, WezTerm, foot, recent VTE, Windows
Terminal), braille otherwise, ASCII for `TERM=dumb` and non-UTF-8 locales.

## Theme

Colors and styles as a value you pass, never a global. Today: the layer palette, with
dark and light variants and `COLORFGBG` detection; role colors and cell aspect ratio
join later. Maps to `Theme` (a field of `Frame`).

## Grid

Small multiples: independently rendered plots pasted side by side
(escape-aware padding), rows filled left to right. Axis sharing is a composition —
fix domains with `Plot::x_domain`/`y_domain` — never a hidden mode. Maps to
`plot::Grid` (re-exported at the root).

## Preset

A plain function composing the grammar into a named chart type: `line()`, `hist()`,
`scatter()`, …. Every preset is provably equal to its grammar expansion (asserted
bit-identical in tests). Presets are the front door; the grammar is discovered, not
required. Maps to root functions — currently `malevich::line`.

## Stream

Live data machinery, kept at the edge of the crate: `stream::Ring` (a sliding window
shared across threads — the one lock in the library; producers push, the render side
snapshots), `stream::Rate` (counters into deltas), and `stream::Live` (in-place
repaint: cursor up, erase down, one buffered write — flicker-free, scrollback-safe,
never owning the screen). The core stays pure; only this module knows time passes.

## Gap

Missing data, encoded as `NaN` in a series and rendered as a visible break — never
interpolated across, never dropped silently. The de-facto convention of the terminal
plotting field.
