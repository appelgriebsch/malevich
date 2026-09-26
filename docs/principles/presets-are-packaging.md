# Presets are packaging

A preset is a name for a grammar expansion, proven byte-identical to it. The
front door and the grammar are the same library.

## Why

Ship `hist()` as its own code path and the grammar has forked. The preset
picks up private options. The expansion drifts from the name. Soon there are
two ways to draw a histogram, and they disagree in the corners. One is for
beginners. One is for people who read the source. The catalog then grows by
accretion. Every chart someone asks for becomes a new function, with its own
defaults. The vocabulary stops being learnable, because knowing the grammar
no longer tells you what the presets will do.

The other failure runs the other way. No front door at all. A grammar-only
library makes the first plot a lesson, and the first look at data should not
require one.

## The idea

Every preset is a plain function. It composes public grammar — marks, stats,
scales — into a named chart, and a test asserts that the rendered output is
byte-identical to the explicit composition. The preset can never
do anything the grammar cannot. It is packaging, not a second code path, and
not different math.

Presets are the front door. The grammar is discovered, not required. `line()`
is the first call. `Plot::new().layer(Line::y(...))` is the same call with
the lid off. Graduating from one to the other changes nothing about the
output. Grouped scatters, volcano plots, Manhattan plots, and candlesticks
never become presets. Each is a few lines of grammar, and the gallery shows
the lines.

Configuration follows the same rule. A `_with` variant takes an options value
and returns a typed error for invalid data or invalid options. Its default
options must reproduce the plain preset exactly. No option exists that only a
preset can reach.

## Consequences

- A preset costs one function and one equality test. It never costs a render
  path.
- The expansion is the documentation. The test keeps that documentation true.
- You graduate in steps: the preset, the preset plus builder calls, then the
  full grammar. There is no cliff, because there is nothing behind the preset
  to learn.
- A chart the grammar cannot spell is a grammar question, not a request for a
  preset. See [What earns a concept](what-earns-a-concept.md).
- The gallery can label a chart "from the grammar, no preset" and mean it.
  That is the strongest evidence the vocabulary suffices.

## Not this

- A preset does not get a private mark, a private stat, or a hidden default
  the grammar cannot express.
- Options objects do not grow, per chart type, into a config kitchen sink.
- No chart-type zoo: one exported function per paper figure.
- "Close enough" is not equality. The test is byte equality of rendered
  output, not visual similarity.

See [What earns a concept](what-earns-a-concept.md) for what may grow the
grammar instead, and [Vision](../vision.md) rule 2.

## Witness

The `hist` preset and its expansion, rendered by the same program that splices
this file. The example asserts the two strings are equal, then prints one of
them:

<!-- generated:witness_packaging -->
```text
hist(&samples) == Bins::auto + Bars::spans + Scale::Integer, byte for byte:
90 ┤                    ▄▄▄▄▄▂▂▂▂▂
   │                    ██████████
60 ┤               ▁▁▁▁▁██████████▁▁▁▁▁
   │          ▁▁▁▁▁████████████████████▄▄▄▄▄
   │          ██████████████████████████████
30 ┤          ██████████████████████████████
   │          ██████████████████████████████
 0 ┤     ▇▇▇▇▇███████████████████████████████████
   └┬─────────┬─────────┬─────────┬─────────┬─────────┬
    0         2         4         6         8        10
```
<!-- /generated -->

## Spelled today

The presets are re-exported at the crate root: `line`, `scatter`, `bar`,
`sparkline`, `hist`, `stairs`, `ecdf`, `heatmap`, `hist2d`, `density`,
`box_plot`, `violin`, `error_bars`, `error_bars_asymmetric`, `trend`,
`contour`, `contourf`, `quiver`, `table`, `describe`, their `_with` twins,
and the checked `try_table`. The equality tests are the
`the_*_preset_equals_its_grammar_expansion` family in
`src/plot/tests/plot_tests.rs`. `witness_packaging` is the spliced example
above. This section may rot; the rest must not.
