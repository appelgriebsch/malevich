# The axes are the product

The parts other libraries get bored of are the product. Ticks are computed.
Labels are exact. If the axes are wrong, nothing else matters.

## Why

Terminal plotting libraries compete on marks and lose on axes. The usual axis
is an afterthought. Ticks sit on arithmetic intervals and land on ugly
values. Labels go through a float formatter and come out
`0.30000000000000004`. The y-axis changes width from row to row. Log scales
and calendar time get no real answer. The body of the chart can be beautiful
and the chart is still unreadable, because the axes are how a reader attaches
numbers to ink.

Label lies are the quiet version of the same failure. A label that rounds
differently from its neighbor, or that shows a value the tick does not sit
on, puts the wrong number in someone's head. They believe it.

## The idea

Treat the boring parts as the product. Tick placement is the extended
Wilkinson algorithm (Talbot, Lin, Hanrahan 2010). Candidate steps are scored
for simplicity, coverage, density, and legibility. That is the placement the
visualization literature settled on, not a heuristic that happens to work at
one size.

Labels are exact decimals. An integer mantissa times a power of ten, written
so every label parses back to exactly its value. Float artifacts are
structurally impossible, not filtered out afterward. One fraction width and
one SI prefix per axis (`2.5M`, `100µ`), so a column of labels lines up and
the reader carries one unit, not a new one each row. Log axes get superscript
decades. Calendar axes get multi-scale labels, and they say `14:05`, or
`Aug 2`, or `2027`, as the span demands. Band axes fit category labels to
their bands.

Layout gets the same care. Labels are measured in display cells, CJK
included. Gutters are computed. Collisions are resolved by shedding furniture,
not by overlapping ink.

## Consequences

- Ticks are never supplied as strings. There is no API for hand-placed ticks,
  and no tick callback, because computed placement is a feature, not a
  limitation.
- Every label round-trips. Parse it and you have the tick's exact value.
- Axis quality is testable. Placement and formatting are pure functions, with
  snapshot coverage, not a judgment call about the picture.
- Log, time, and band scales are first-class axis types, not label formatters
  bolted onto a linear scale.
- The formatter is shared. Colorbars, legends, and the notes that disclose
  quantization speak the same exact decimals as the axes.

## Not this

- `format!("{}", 0.1 + 0.2)` does not get near a label.
- Tick counts do not ignore the space they have, and labels do not overlap.
- Caller-supplied tick strings are not the escape hatch for bad placement,
  and neither is a tick callback.
- Small charts do not get a second, cheaper formatter.

See [The full draw is the oracle](full-draw-oracle.md) for the ink half of
honesty, and [Vision](../vision.md) rule 3.

## Witness

Ticks stepping by 0.2, a value with no exact binary form. That is the classic
float-artifact trap. Spliced here by the doc generator. Every label is an
exact decimal, and each axis shares one fraction width:

<!-- generated:witness_axes -->
```text
             every label an exact decimal
0.6 ┤     ⢀⠔⠉⠉⠉⠑⠤⡀                           ⡠⠒⠉⠉⠒⠒⡄
    │    ⡔⠁      ⠈⢆                        ⡠⠊      ⠈⠑⢄
    │  ⡠⠊          ⠑⢄                     ⡜          ⠈
0.4 ┤ ⡜             ⠈⢆                  ⢀⠎
    │⠜                ⠣⡀               ⡰⠁
0.2 ┤                  ⠱⡀            ⢀⠔⠁
    │                   ⠈⢆          ⢀⠎
    │                    ⠈⠢⣀      ⢀⠔⠁
0.0 ┤                       ⠣⢄⣀⣀⡠⠔⠊
    └┬───────┬───────┬────────┬───────┬───────┬───────┬
     0      10      20       30      40      50      60
```
<!-- /generated -->

## Spelled today

`scale::Ticks` is extended-Wilkinson placement, with `Ticks::log10` decades
and `Ticks::time` calendar ticks over unix seconds; `scale::Scale` is the
axis specification (`Linear | Log | Time | Bands`). Exact-decimal formatting
and per-axis SI prefixes live in the tick formatter; label measurement is
display-cell aware via `unicode-width`. This section may rot; the rest must
not.
