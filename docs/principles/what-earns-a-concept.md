# What earns a concept

A concept sits in the vocabulary only when real charts demand it and no
composition of the rest can draw it. Both clauses. Everything else is a
preset or an example.

## Why

Chart libraries die of vocabulary. Someone asks for a chart, and it lands as
a new type. The type grows its own options. A few years on, the library is a
catalog of a hundred near-duplicates. `barh` beside `bar`. `Heatmap` beside
`Image` beside `Matrix`. None of them compose. All of them disagree in small
ways. Each one is a maintenance bill that does not end. The cost to the user
is worse than the maintenance cost. A vocabulary that large cannot be learned.
It can only be searched.

The opposite failure is purism. Refuse a concept the grammar genuinely cannot
express, and people hand-roll the hard part. The hand-rolled versions get the
statistics wrong. That is how a plotting ecosystem ends up with a hundred
incorrect quartile implementations.

## The idea

One membership test. Two clauses, both required.

- Real charts demand it. Charts people actually draw, not charts a
  completeness argument names.
- No composition of the existing marks, stats, and scales reproduces its
  rendered output.

The eight marks pass. A violin is composable from `Area` once the KDE exists,
but the KDE itself is not composable from marks, so the stat earns a seat and
the violin stays a preset. `Cells` earns one mark, not three. Value grids, rgb
images, and categorical regions are one geometry with three color readings. A
heatmap is not a mark at all. It is `Cells` under a colormap, and the preset
says so.

The same test shapes the statistics layer. One `Reducer` vocabulary serves
bins, groups, and windows, so "rolling p95" needs zero new concepts. The layer
still refuses a fake uniform algebra. Online accumulators, reducers, and
batch transforms keep distinct execution contracts. Pretending every statistic
merges is how libraries ship wrong parallel medians.

A new concept must also pay for itself *across* features. The band scale
earned its seat by serving bar charts, confusion matrices, and attention maps
with one mechanism. A concept that serves one chart is that chart's
implementation detail, not vocabulary.

## Consequences

- The mark family is closed, and declared complete. Adding a mark is a design
  event, judged by the test, never a convenience.
- A feature request gets a composition first. "From the grammar, no preset"
  in the gallery is the test passing in public.
- Options do not multiply. An option must be a mark channel, a stat
  parameter, a scale option, or a theme entry, or it does not ship.
- Statistical correctness lives in one place. One type-7 quantile
  implementation serves the box plot, the reducers, and the Q–Q plot.
- Removing is a contribution. A concept whose charts the grammar learns to
  compose is retired at the next major version.

## Not this

- No mark per chart type, and no `Heatmap` mark beside `Cells`.
- "We might need it" is not clause one. "It would be elegant" is not clause
  two.
- No uniform `Stat` trait that pretends every statistic streams and merges.
- The mark enum does not open so dependents can register marks. New geometry
  lands in the crate, under this test.

See [Presets are packaging](presets-are-packaging.md) for where refused
concepts go instead, and [Vision](../vision.md) rule 2.

## Spelled today

`mark::Mark` is the closed enum over `Line`, `Points`, `Bars`, `Area`,
`Cells`, `Range`, `Rule`, `Text`; its docs declare the family complete.
`stat::Reducer` is the shared aggregation vocabulary; the execution
contracts are documented per type in `stat` (accumulators with merge laws,
reducers, batch transforms). `scale::Scale` is the closed axis
specification. The refusals are recorded in the README's "What it will not
be." This section may rot; the rest must not.
