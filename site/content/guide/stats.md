# The statistics layer

A stat is a data operation that runs before the scales see the data. The word follows seaborn's objects interface and ggplot's `stat_*`, and it is the module-level umbrella, not one execution algebra: a stat may be an online accumulator, a reducer, keyed orchestration, or a batch transform. The layer refuses a fake uniform `Stat` trait, because pretending every statistic streams and merges is how libraries ship wrong parallel medians.

Everything here lives in `malevich::stat`. This page walks the module with a plate per idea.

## Bins and histograms

`Bins` is a histogram accumulator: a start, a width, a count, and the counts. `Bins::auto(values, limit)` chooses the geometry — the Freedman–Diaconis width from type-7 quartiles, the whole finite span binned whole, at most `limit` bins — and `add` streams values in. `Bars::spans` draws it.

{{figure stat_bins}}

That is the entire `hist` preset, and the packaging test asserts it. `HistogramOptions` adds what a histogram sometimes needs: a bin cap, a `Normalization` (count, probability, percent, density per unit of x), and `cumulative`. `kaz hist --normalize percent --cumulative` is the same options value from the shell.

{{figure stat_bins_normalized}}

`bins2` is the two-dimensional form behind `hist2d`, and `calendar_bins` counts unix timestamps per hour, day, ISO week, month, or year — buckets of their true length with empties kept — feeding `Bars::intervals`.

{{figure stat_calendar}}

## Densities

`kde(values, points)` is a Gaussian kernel density estimate at Silverman's bandwidth. `kde_with` takes `KdeOptions`: a `Bandwidth` (Silverman, a scale of it, or a fixed width), `bounds` that reflect the kernels so a latency density stays above zero, a `cut` in bandwidths past the data, and the cumulative form. The bandwidth is the one honest knob a density has, and the plate shows why it is worth showing three.

{{figure stat_kde}}

The `density` and `violin` presets are built on it; the gallery's [latency](../../gallery/#latency) example shows the bounded estimate beside the unbounded one that leaks below zero.

## Quartiles, whiskers, and moments

`BoxStats::of(values)` computes type-7 quartiles (Hyndman and Fan's estimator, the one R and NumPy default to), Tukey whiskers at 1.5 IQR, and the outliers beyond them. `Whiskers::Percentiles(lo, hi)` and `Whiskers::MinMax` are the other rules; `BoxOptions::whiskers` passes one to the preset.

{{figure stat_box_whiskers}}

`Moments` is the online accumulator behind `describe`: count, mean, variance, standard deviation (population and sample), min, max — updated one observation at a time, and mergeable, because its summary state is order-independent. `quantiles(values, positions)` is the type-7 estimator on its own, shared by the box plot, the reducers, and the Q–Q plot, so there is exactly one quartile implementation in the crate.

{{figure stat_describe}}

## The reducer vocabulary

One `Reducer` serves every aggregating stat: `Count`, `Sum`, `Mean`, `Median`, `Min`, `Max`, `Percentile(q)`, `Deviation`, `Variance`, `StdErr`, `First`, `Last`. Bins, groups, windows, and dense grids all take one, so a rolling p95, a binned median, a group's mean ± standard error, or a max-reduced heatmap is one call and zero new concepts.

`Window` is the rolling form: a size, an anchor (trailing by default, or `Middle`, or `Start`), and `strict`, which gaps the positions whose window is incomplete instead of guessing.

{{figure stat_window}}

{{figure stat_window_anchor}}

`binned(x, y, &bins, reducer)` reduces y over the bins of x — the reliability diagram of a classifier is `Reducer::Mean` over 0/1 outcomes binned by confidence.

{{figure stat_binned}}

`Agg::by(keys, values)` groups by a categorical key and reduces each group; `Cells::reduce` applies the same vocabulary to grids denser than the raster.

{{figure stat_agg}}

## Smoothing, explicitly

`ewma(values, alpha)` is a debiased exponential moving average. `lttb(x, y, target)` is Largest-Triangle-Three-Buckets. Both are *inexact* transforms, and both exist only as explicit stats the caller applies — never as a silent default reduction. The default reduction is M4, below, which is exact.

{{figure stat_ewma}}

{{figure stat_lttb}}

## Fits

`Fit` is streaming least squares: `add(x, y)` or `Fit::xy(&x, &y)`, then `slope`, `intercept`, `r_squared`, `predict(x)`, `standard_error(x)`. It merges — two halves of a dataset fitted separately combine exactly — because its summary state is order-independent. The `trend` preset draws the scatter, the line, and a 95 % confidence band from one accumulator.

{{figure stat_fit}}

## Distributions as curves

`ecdf(values)` is the empirical cumulative distribution as a step; the `ecdf_with` preset adds the Dvoretzky–Kiefer–Wolfowitz band at a chosen level. `roc(scores, labels)` sweeps the thresholds and `auc(x, y)` integrates the result.

{{figure stat_ecdf}}

{{figure stat_roc}}

## Stack and dodge

`stack(&[&a, &b, &c])` returns a `(low, high)` pair per series, positives above the baseline and negatives below it, for `Area::between` or `Bars::base`. `stack_with` takes `StackOptions`: `StackOffset::Normalize` is the 100 % stack, `Center` the streamgraph silhouette, and `StackOrder::Sum` piles the largest series first.

{{figure stat_stack}}

{{figure stat_stack_normalize}}

`dodge(&[&a, &b], step)` is stack's sibling for bars beside each other: side-by-side positions within each band, one series per value series, fed to `Bars::at`. Grouped bars are therefore a composition, never a preset — the presets principle applied to the request the field files most.

{{figure stat_dodge}}

## Positions and shapes

`jitter(positions, width)` spreads a strip of points evenly across its band with van der Corput offsets: no seed, the same every time, no clumps. `steps(x, y, direction)` is the piecewise-constant expansion the `stairs` preset draws, changing after, before, or midway between samples. `nearest` is the crosshair-snapping lookup interactive hosts use: the index of the closest finite value, so a readout shows a datum that exists rather than an interpolation.

{{figure stat_jitter}}

{{figure stat_steps}}

## The series maps

`cumsum`, `diff`, `rank`, and `normalize(values, reducer)` — a running total, first differences, ranks, and division by a reducer of the whole series. Index charts and percent-of-peak are one call.

{{figure stat_maps}}

## Contours

`contours(values, columns, levels)` runs marching squares over a grid and returns iso-lines as paths. The `contour` preset chooses its levels the way an axis chooses ticks, and `contourf` is `heatmap` under a colormap split at those levels.

{{figure stat_contours}}

## M4: the reduction that is not a stat

`m4(x, y, columns)` keeps the first, last, minimum, and maximum point of every raster column. Because points are bucketed by the column they render into, the reduction reproduces the full draw's pixels exactly (Jugel et al., PVLDB 2014). The plot inserts it automatically past four points per column; it is public so a host can pre-reduce a series it will render many times.

{{figure stat_m4}}

The alternative every sampling library ships, drawn honestly for comparison — the same series, every 400th point:

{{figure stat_stride}}

Three one-sample spikes are gone, and nothing on the chart says so. The oracle test in the crate asserts raw raster equals reduced raster at several frame sizes; a reduction change that breaks it is a different chart, not an optimization ([the full draw is the oracle](../../principles/full-draw-oracle/)).

## Execution contracts

The stats keep distinct contracts, stated per type:

| kind | examples | contract |
|---|---|---|
| online accumulator | `Moments`, `Fit`, `Bins`, `M4` | bounded state, one observation at a time; `Moments` and `Fit` merge in any order, `Bins` needs identical geometry, `M4` needs chunks in series order |
| reducer | `Reducer::*` | a result for one collection; no public merge |
| batch transform | `Window`, `kde`, `ecdf`, `roc`, `ewma`, `lttb`, `stack`, `contours`, `bins2`, `BoxStats` | consumes a complete ordered collection, emits another; may use accumulators inside, which does not make it mergeable |

Merge results are understood over a fixed reduction tree, not as bitwise-independent reassociation. Every type states its own identity and preconditions in its docs.
