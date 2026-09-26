# Scales and axes

The differentiators live exactly where everyone else got bored. A scale maps a data domain to a raster range with the d3-scale contract — `nice`, `ticks(n)`, `invert`, a formatter — and the axis it draws is treated as the product: ticks are computed by the placement the visualization literature settled on, and labels are exact decimals that parse back to their values. If the axes are wrong, nothing else matters ([why](../../principles/axes-are-the-product/)).

## Five position scales

The axis specification is `Scale`, set with `Plot::x_scale` and `Plot::y_scale`; `Auto` is the default and infers from the layers.

| scale | what it is | shortcut |
|---|---|---|
| `Linear` | the ordinary axis | — |
| `Integer` | linear, but the tick step never drops below one | `hist` uses it for counts |
| `Log` | decades, superscript labels, non-positive values as gaps | `log_x()`, `log_y()` |
| `Time` | unix seconds with calendar labels | `time_x()` |
| `Bands(categories)` | one band per category; the bar-family axis on x, matrix rows on y | `Scale::bands(...)` |

{{pair scale_linear scale_integer | The same four counts on a tall frame. A linear axis labels 0.5 and 1.5; `Scale::Integer` never labels a half of anything.}}

{{figure scale_log}}

{{figure scale_bands_y}}

## Calendar time

A time axis labels what the span demands: `14:05` across a session, `Aug 2` across a month, `2027` across decades. What the labels leave out, the axis prints once — the date under hour labels, the year under day labels — as a context note at the end of the axis-title row. That is an automatic layout rule, not an option, and it sheds with the title row when the frame is small.

{{figure scale_time_hours}}

{{figure scale_time_days}}

{{figure scale_time_years}}

## Ticks

Tick placement is the extended Wilkinson algorithm (Talbot, Lin, Hanrahan 2010): candidate steps are scored for simplicity, coverage, density, and legibility, and the labels are measured in display cells — CJK included — so the search knows what fits. There is no API for hand-placed ticks, because computed placement is a feature, not a limitation.

Labels are exact decimals: an integer mantissa times a power of ten, formatted so every label parses back to exactly its value. Float artifacts are structurally impossible, not filtered out. One fraction width and one SI prefix per axis (`2.5M`, `100µ`), so columns align and the reader carries one unit, not one per row.

{{figure scale_ticks}}

Values that agree in their leading digits read relative to a round base the axis prints once — matplotlib's offset text, done as a note rather than a surprise:

{{figure scale_context}}

`NumberFormat` is the same decision procedure for an arbitrary set of related values — one fraction width, one prefix, whole labels for whole-number sets, gaps as `—` — and it is the per-column formatter behind `table` and `describe`. There is no second, cheaper formatter anywhere in the crate. A value the set's resolution would misstate — one that would round to zero — keeps its own resolution, so a column of gigabytes never reads a mean of a thousand as `0`.

## Domains

A domain is a scale option. `x_domain(lo, hi)` and `y_domain(lo, hi)` fix both ends exactly, matplotlib-style; `x_min`, `x_max`, `y_min`, `y_max` fix one end while the other fits the data and grows to its outer tick.

{{pair scale_domain_auto scale_domain_fixed | Automatic and fixed. What falls outside a fixed domain is clipped: drawn nowhere, never moved onto the border. A point at the edge is a point at the edge, not a refugee from beyond it.}}

{{figure scale_one_sided}}

Interactively, a `Viewport` is the same two windows as a value a host can zoom, pan, clamp, and tail — which is why zooming into millions of points needs no special machinery: the reduction re-aggregates to the new window ([interaction](../interaction/)).

## Units

A linear or integer axis may carry a `Unit`, set with `x_unit` and `y_unit`. The ticks stay the same ticks; only the labels change, and the `Mapping` readout speaks the same unit.

{{pair scale_unit_si scale_unit_bytes scale_unit_suffix | `Unit::si("B")` puts the axis's one SI prefix before the unit; `Unit::Bytes` chooses ticks that are nice in KiB and MiB; `Unit::suffix("%")` appends a bare suffix and never a prefix.}}

## Colormaps

`Colormap` is the color scale for `Cells`, `Line::grade`, `table_with`, and the colorbar. Six curated ramps — the sequential four are perceptually uniform, the diverging two are balanced — and stops mix in OKLab, so the color halfway between two stops looks halfway: no grey between blue and yellow.

<div class="two-up">

{{figure cm_viridis nocode}}

{{figure cm_magma nocode}}

{{figure cm_cividis nocode}}

{{figure cm_greys nocode}}

{{figure cm_red_blue nocode}}

{{figure cm_purple_orange nocode}}

</div>

A colormap has four options, each a scale option rather than a mode:

{{figure scale_colormap_centered}}

{{figure scale_colormap_log}}

{{figure scale_colormap_domain}}

{{figure scale_colormap_steps}}

Positions clip; colors squish. A value outside a fixed color domain is pushed into the ramp's nearest end, because a cell must be some color — and `under` and `over` disclose the squish with colors of their own, while the colorbar shows the fixed range. `thresholds(values)` splits the ramp at explicit boundaries; `contourf` is `heatmap` under a map split at `contour`'s levels.

## Palettes

`Palette` is the categorical scale `color_by` draws from, and it lives in the spec — a serialized plot keeps its category-to-color assignment so its legend means the same thing wherever it renders. The default is Okabe–Ito (Wong 2011), colorblind-safe; Paul Tol's `BRIGHT` and `MUTED` sit beside it. `Plot::palette` chooses; `Palette::new` takes your own colors.

{{figure palette_okabe_ito}}

{{figure palette_bright}}

{{figure palette_muted}}

The `Theme`, by contrast, is a frame property: the colors layers take when they set none, adapted to a dark or a light background. Two palettes, two homes, one recorded reason — presentation adapts to the terminal; an encoding travels with the data.

## What an axis will not do

- Accept caller-supplied tick strings, a manual tick list, or a format callback. Ticks are computed and their labels are exact; text a caller writes goes in a `Text` mark.
- Print `0.30000000000000004`, or `-0`, or 309 digits. Every path through the formatter is the exact-decimal one, including the fallback for a span no nice step can cover.
- Draw a second y axis. Two series on two scales in one panel lie about their relative magnitude; two panels of a `Grid` sharing one x window compare them honestly ([composition](../composition/)).
- Interpolate across a gap, smear an out-of-range point onto the border, or clamp a non-positive value onto a log axis. A gap is a break; out of range clips; log of nothing is a gap.
