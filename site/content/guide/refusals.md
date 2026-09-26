# What it will not be

Not a TUI framework. It never owns the terminal, and it never handles input. No animations. No file parsing or dataframes in core. Ingestion is traits only. No config-object kitchen sink. If an option is not a mark channel, a stat parameter, a scale option, or a theme entry, it does not ship. No general table widget. A malevich table is a statistical summary it computed and formatted. Borders, spans, wrapping, and cell styling belong to table crates.

The refusals the field asks about most. Each has its reason, and the answer that already exists.

## No pies, donuts, polar, radar, or 3D

The grammar is closed at eight marks over Cartesian scales. A pie encodes parts by angle. The eye ranks that poorly, and a cell grid cannot draw it. The honest forms read the parts against a straight axis. A pie is a `Cells::classes` waffle — one hundred cells the eye can count — or a `Bars` breakdown under `StackOffset::Normalize`.

{{example waffle}}

A ridgeline — rows rendered back to front at fixed elevation — is the terminal's honest 3D surface, and a `Cells` heatmap is a surface seen from above.

## No twin y axes or axis breaks

A second scale in one panel lets two series lie about their relative magnitude. An axis break lets a bar lie about its length. Two panels of a `Grid` sharing one x window put two series side by side honestly, each with its own true y ([composition](../composition/#two-series-two-scales)).

{{pair comp_shared_a comp_shared_b}}

## No tick-format callbacks, manual tick lists, or thousands separators

Ticks are computed by extended Wilkinson placement, and their labels are exact decimals. There is no hook to make them wrong. A unit is a scale option (`y_unit`). A base the labels share is printed once as a context note. Text a caller writes goes in a `Text` mark, not on the axis ([scales and axes](../scales/#ticks)).

{{figure scale_context nocode}}

## No interpolation across gaps, no smearing, no log clamping

A gap is a visible break. An out-of-range position clips — drawn nowhere, never moved onto the border. A non-positive value on a log axis is a gap. Colors are the one exception, disclosed as such. A value outside a fixed colormap domain squishes into the ramp's nearest end, unless `under` and `over` name it, because a cell must be some color.

{{figure start_gap nocode}}

## No sampling as a default reduction

The default reduction of a large line is M4, which reproduces the full draw pixel for pixel. Striding, "one point per bucket", and LTTB are not defaults. `stat::lttb` and `stat::ewma` exist as explicit, opt-in, inexact transforms the caller applies on purpose ([the statistics layer](../stats/#m4-the-reduction-that-is-not-a-stat)).

{{pair stat_m4 stat_stride | M4 on the left keeps the three spikes by construction. The sampler on the right lost them and cannot say so.}}

## No key polling or gesture configuration

Interaction is arithmetic over a `Viewport` the host feeds. The host owns its input. The ratatui and Ink widgets interpret a fixed default gesture set from coordinates the host maps. A host that wants different policy drives `Viewport` and `Mapping` directly, the same escape hatch presets give the grammar ([interaction](../interaction/)).

## No probes where escapes are unsafe

Detection sniffs the environment anywhere. It probes the terminal only where the destination is a tty, nothing sits between, and `TERM` is not dumb. An unanswered probe is not evidence. Redirected output can never contain interrogation escapes ([frames and terminals](../terminals/)).

## No dense charsets by default

A terminal name can't prove the configured font covers octants, sextants, or braille. `Frame::detect` picks quadrants in any UTF-8 environment and ASCII otherwise. The dense tiers are choices you make for fonts you know, through `MALEVICH_CHARSET` or an explicit frame.

## No global state

No global theme, palette registry, or default size. A plot holds no writer. A render consults no environment variable. The three documented conveniences that read the environment — `Display`, `Frame::detect`, `render_best` — construct values that then drive pure calls ([the frame is run state](../../principles/frame-is-run-state/)).

## Removing is a contribution

A concept whose charts the grammar learns to compose is retired at the next major version. The vocabulary is judged by one test: real charts demand it, and nothing else in the grammar can draw it. A library that passes that test stays small enough to learn ([what earns a concept](../../principles/what-earns-a-concept/)).
