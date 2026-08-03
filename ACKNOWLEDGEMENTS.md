# Acknowledgements

malevich stands on the shoulders of giants. Before a line of it was written, we studied
the terminal-plotting field across five language ecosystems — the libraries below are
not a courtesy list; each one taught this project something specific, named here.

## Algorithms

- **Justin Talbot, Sharon Lin, Pat Hanrahan — "An Extension of Wilkinson's Algorithm
  for Positioning Tick Labels on Axes"** (IEEE InfoVis 2010,
  [vis.stanford.edu/papers/tick-labels](http://vis.stanford.edu/papers/tick-labels)).
  Our axis ticks are this algorithm, built on **Leland Wilkinson**'s original labeling
  work and his *The Grammar of Graphics* (Springer, 2005) — the book behind our whole
  marks-and-scales vocabulary.
- **Jack Bresenham** — the line rasterization algorithm (IBM Systems Journal, 1965)
  running inside every polyline we draw.
- **You-Dong Liang and Brian Barsky** — the parametric clipping (1984) that lets our
  surface promise "drawing never fails".
- The Unicode Consortium's **Braille Patterns** block (U+2800–U+28FF), the quiet
  workhorse of sub-cell terminal graphics.
- **Daniel Hepper's [font8x8](https://github.com/dhepper/font8x8)** (public domain,
  after Marcel Sondaar and the IBM PC BIOS fonts) — the 8×8 bitmap font baked into
  the `pixel` feature so in-panel text can render without a font stack.
- **DEC's sixel format** (VT340, 1987) and **kitty's graphics protocol** (Kovid
  Goyal) — the two ends of the terminal-image lineage our pixel encoders speak,
  with the transparency-by-omission trick (`P2=1`) learned from how notcurses and
  chafa handle backgrounds.

## Terminal plotting — the tradition we join

- **[UnicodePlots.jl](https://github.com/JuliaPlots/UnicodePlots.jl)** (Julia) — the
  most complete terminal plotting library anywhere, and our architectural north star.
  Its canvas family, its text-and-pixels-share-one-grid rule, and its demonstration
  that many plot types reduce to a few raster primitives shaped our `Surface` and
  `Charset` directly. We studied its source line by line.
- **[drawille](https://github.com/asciimoo/drawille)** — the original braille canvas;
  every braille chart in every language descends from it.
- **[textplots](https://github.com/loony-bean/textplots-rs)** (Rust) — braille plotting
  in Rust before us, and the source of one idea we adopted whole: sampling a function
  at raster resolution.
- **[asciigraph](https://github.com/guptarohit/asciigraph)** (Go) and
  **[rasciigraph](https://github.com/orhanbalci/rasciigraph)** (Rust) — the
  box-drawing line-chart lineage, the NaN-as-gap convention, and the lesson that the
  label gutter must be measured, never fixed.
- **[asciichart](https://github.com/kroitor/asciichart)** (JavaScript) — the famously
  elegant ~100-line `╭╮╰╯` renderer, ported to a dozen languages; a planned line style
  here.
- **[plotext](https://github.com/piccolomo/plotext)** (Python) — proof of how much a
  terminal plotting library can cover, and an honest study in the costs of global
  state.
- **[YouPlot](https://github.com/red-data-tools/YouPlot)** (Ruby) and
  **[ttyplot](https://github.com/tenox7/ttyplot)** (C) — the command-line and
  streaming UX our future CLI will learn from.
- **[gnuplot](http://www.gnuplot.info)** — plotting in terminals since 1986; its
  `dumb` and `block` terminals are the ancestors of this whole field.
- **[ntcharts](https://github.com/NimbleMarkets/ntcharts)** (Go) and **Rich /
  [Textualize](https://github.com/Textualize/rich)** (Python) — the render-to-a-string,
  never-own-the-terminal principle we hold as a core value; Rich's maintainer stated
  it as policy, ntcharts proved it in practice.
- **[lowcharts](https://github.com/juan-leon/lowcharts)** (Rust) — the two-phase
  fix-bounds-then-bin aggregation shape our statistics layer builds on.

## API design — the grammars we learned from

- **[d3](https://github.com/d3/d3-scale)** — the scale abstraction
  (domain → range, `nice`, `ticks`, `invert`) is the best-designed scale API anywhere;
  ours follows its contract.
- **[Observable Plot](https://github.com/observablehq/plot)** — the proof that a small
  orthogonal basis (marks × scales × transforms × facets) covers the chart catalog,
  and the model for zero-config defaults done right.
- **[Vega-Lite](https://vega.github.io/vega-lite/)** — the serializable declarative
  spec as a value, which is what our retained `Plot` is.
- **[seaborn](https://seaborn.pydata.org)**'s objects interface — the Mark + Stat
  split that makes our aggregation pipeline a composition rather than a special case.
- **[uPlot](https://github.com/leeoniya/uPlot)** — columnar data, allocation
  discipline, and the practice of publishing a non-goals list; we keep one because
  uPlot showed why.
- **[pillar](https://github.com/r-lib/pillar)** (R) and
  **[tidy-viewer](https://github.com/alexhallam/tv)** (Rust) — the significant-figures
  formatting semantics our number formatting follows.
- **[matplotlib](https://matplotlib.org)** — the basic plot-type catalog we treat as a
  coverage checklist, and four decades of accumulated wisdom about what a plotting
  library owes its users.

## The Rust neighborhood

- **[ratatui](https://github.com/ratatui/ratatui)** and
  **[tui-widgets](https://github.com/joshka/tui-widgets)** — the terminal-UI platform
  we integrate with rather than compete with, and the source of widget-API
  conventions (consuming builders, snapshot-tested buffers, charset tables as data)
  we follow.
- **[terminal_size](https://crates.io/crates/terminal_size)** — our one runtime
  dependency, and **[criterion](https://crates.io/crates/criterion)**, which keeps our
  performance claims honest.

## The name

**Kazimir Malevich** painted *Black Square* in 1915: a small vocabulary of geometric
forms, composed deliberately, on a plain ground. That is the design budget of this
library, and the reason release 0.1.0 carries the painting's name.
