# Terminology

The codebase contract: every public concept is named here before it is named in code.
When a concept is added, renamed, or changes meaning, this file is updated in the same
change. Each entry: what the word means in the wider literature, why this word, and what
it maps to in the crate. Types marked *(planned)* do not exist yet.

## Plot

The retained description of a chart: layers plus scales plus furniture (title, labels,
legend). A plain value — cloneable, inspectable, serializable — with no connection to a
terminal. Rendering is a pure function of a `Plot` and a `Frame`. Maps to `Plot`
*(planned)*.

## Layer

One mark bound to data and options, stacked with other layers on shared scales. The
layering concept of every grammar of graphics (Wilkinson 2005; Vega-Lite `layer`;
UnicodePlots `lineplot!`). Maps to `Plot::layer` *(planned)*.

## Mark

A family of geometric primitives that draw data: `Line`, `Points`, `Bars`, `Area`,
`Cells`, `Range`, `Rule`, `Text`. The word follows Observable Plot and Vega-Lite
("mark"), chosen over matplotlib's "artist" (too broad) and "geom" (ggplot jargon).
Chart types are compositions of marks, never peers of them. Maps to the `mark` module
*(planned)*.

## Channel

A per-mark visual variable fed from data or set constant: `x`, `y`, `y2`, `color`,
`label`, …. Follows Vega-Lite/Observable Plot "encoding channel". Channels accept
anything series-shaped (see Series). Maps to mark constructor arguments and builder
methods *(planned)*.

## Series

One column of scalar data after ingestion: contiguous `f64`, where `NaN` is a gap (see
Gap). The ingestion boundary is the `IntoSeries` trait — slices, iterators, and
closures convert exactly once at the rim; the core is monomorphic `f64`. Maps to
`Series` and `IntoSeries` *(planned)*.

## Stat

A data transform that runs before scales see the data: `Bin`, `Agg`, `Window`, `Stack`,
`Normalize`, `Density`, `Ecdf`, `BoxStats`, `Downsample`, `Contour`. The word follows
seaborn.objects (`Stat`) and ggplot (`stat_*`). Stats are mergeable: two partial results
combine associatively, which is what makes host-side parallelism and streaming
compositions rather than features. Maps to the `stat` module *(planned)*.

## Reducer

A named aggregation shared by every aggregating stat: `count`, `sum`, `mean`, `median`,
`min`, `max`, percentiles. One vocabulary across `Bin`, `Agg`, and `Window` (the
Observable Plot convention). Maps to `Reducer` *(planned)*.

## Scale

A mapping from data domain to raster range with the d3-scale contract: `nice`,
`ticks(n)`, `invert`, and a tick formatter. Position scales: `Linear`, `Log`, `Time`,
`Band`; color scales: sequential, diverging, categorical. Maps to the `scale` module
*(planned)*.

## Ticks

The axis values a scale chooses to label, placed by the extended Wilkinson algorithm
(Talbot, Lin, Hanrahan, InfoVis 2010) — scored for simplicity, coverage, density, and
legibility. Ticks are computed, never supplied as strings, and carry exact-decimal
labels (integer mantissa times a power of ten): labels parse back to their values, share
one fraction width per axis, and never show float artifacts. Maps to `scale::Ticks`.

## Frame

Where and how to render: width and height in cells, charset, color mode, theme. Frame
is render state, not plot state — the same `Plot` renders into many frames.
`Frame::detect()` is the only place the crate inspects the environment. Maps to `Frame`
*(planned)*.

## Surface

The subpixel grid that marks draw on during rasterization, before glyphs exist. A
charset codec maps each cell's subpixel pattern to one glyph plus colors. Internal —
users see strings and widgets, not surfaces. Maps to `render::Surface` *(planned)*.

## Charset

A glyph tier used to encode the surface: `Ascii`, `Blocks`, `Braille`, `Sextants`,
`Octants`, or `Auto` (environment-sniffed, never probed). Glyph tables are data, not
code. Maps to `Charset` *(planned)*.

## Theme

Colors and styles as a value you pass, never a global: palette, role colors (gap,
negative, accent), cell aspect ratio. Maps to `Theme` *(planned)*.

## Preset

A plain function composing the grammar into a named chart type: `line()`, `hist()`,
`scatter()`, …. Every preset is provably equal to its grammar expansion (asserted
bit-identical in tests). Presets are the front door; the grammar is discovered, not
required.

## Gap

Missing data, encoded as `NaN` in a series and rendered as a visible break — never
interpolated across, never dropped silently. The de-facto convention of the terminal
plotting field.
