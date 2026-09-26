# The grammar

A chart library usually grows by adding on: one function per chart type, one option per request. A plot here is written once, as data — layers on shared scales, plus furniture — and every later stage reads that same value. This page builds one chart six layers deep, with a plate at every step, and ends where it started: at the preset that would have drawn the first step in one call.

The data is real: 342 Palmer penguins (CC0), bill length against bill depth.

## One layer

A layer is one mark bound to data. `Points::xy` takes two series. The plot unions their domains, places ticks, and draws.

{{figure grammar_1}}

`Plot::new()` is a value. `.layer(mark)` returns a new value with the layer appended. There is no `Chart::show()`, no figure object, no axes handle to configure. Nothing here knows about a terminal.

## A channel

A channel is a visual variable on a mark. Data can feed it, or you can set it constant. Position channels come through the constructor arguments. Constant channels are builder methods: `color`, `label`, `style`. The data-bound color channel is `color_by`. Categories take palette colors in first-appearance order and name themselves in the legend. In colorless output they cycle marker shapes, so a group never vanishes in a pipe.

{{figure grammar_2}}

Three clusters appear. Note what did not happen: no legend was configured, no palette chosen, no marker shapes assigned. Okabe–Ito is the default palette because it is colorblind-safe. The legend is furniture the plot lays out and sheds when there is no room.

## More layers, and a stat

Each species gets its own least-squares line. `stat::Fit` is a streaming accumulator — feed it pairs, ask for slope, intercept, R², a prediction, a standard error — and the `trend` preset is built on it. Here it is used directly: fit per species, predict at both ends, draw a `Line`.

{{figure grammar_3}}

The lines take the next palette colors in layer order. Layers are independent, and their domains union. Adding one never changes how another is drawn.

## Reference marks

Two marks exist for saying something *about* the data. A `Rule` is a line at one value across the whole plot, horizontal or vertical, optionally dashed and labeled. It also draws spans between two values. A `Text` is a string at data coordinates.

{{figure grammar_4}}

The pooled regression over all three species has a *negative* slope: taken together, longer bills look shallower, because Gentoos have long shallow bills. Within every species the slope is positive. That is Simpson's paradox, and it is the kind of thing a chart exists to show.

## Furniture

Title and axis labels come last, because they are the last thing the layout places and the first thing it sheds when a frame is small. The legend already exists. It came with the labels.

{{figure grammar_5}}

Six layers, one plot value. It is `Clone + Send + Sync`, serializable with the `serde` feature, and renders identically in any frame you hand it.

## Back through the front door

The first step of this page, as a preset:

{{figure grammar_preset}}

`scatter(x, y)` is `Plot::new().layer(Points::xy(x, y))`, and a test asserts the rendered strings are equal byte for byte. Every preset is packaged this way — nothing behind a preset to learn — so the path has no cliff: preset, preset plus builder calls, full grammar ([why](../../principles/presets-are-packaging/)).

## The vocabulary, in one table

| concept | what it is | where it lives |
|---|---|---|
| Plot | the retained description: layers, scales, furniture | `Plot` |
| Layer | one mark bound to data and options | `Plot::layer` |
| Mark | eight geometric primitives: `Line`, `Points`, `Bars`, `Area`, `Cells`, `Range`, `Rule`, `Text` | [the eight marks](../marks/) |
| Channel | a per-mark visual variable: `x`, `y`, `color`, `label`, `color_by`, `align`, … | constructor arguments and builder methods |
| Stat | a data operation before scales see the data: bins, KDE, box stats, fits, windows, stacks, M4 | [the statistics layer](../stats/) |
| Scale | data domain to raster range: `Linear`, `Integer`, `Log`, `Time`, `Bands`; colormaps and palettes | [scales and axes](../scales/) |
| Frame | one rendering's size, charset, color mode, theme | [frames and terminals](../terminals/) |
| Preset | a plain function composing the grammar into a named chart type | the crate root |

The grammar is closed. A feature has to be a mark channel, a stat parameter, a scale option, or a theme entry, or it does not ship. A new concept has to pay for itself across many features. The eight marks are declared done. That is what keeps the vocabulary small enough to learn ([why](../../principles/what-earns-a-concept/)).

## Reading the gallery

Every chart in the [gallery](../../gallery/) is labeled either as a preset or "from the grammar, no preset". The second label is the test passing in public: a raincloud, a volcano plot, a Manhattan plot, a candlestick chart, an annotated confusion matrix, a waffle — each a few lines of the vocabulary above.

{{example raincloud}}
