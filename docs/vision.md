# Vision

Malevich is terminal plotting for Rust: a small grammar of marks, honest axes,
millions of points. A plot is a plain value, and rendering is a pure function
of that value and a frame. The first look at any data should happen in the
terminal, and it should be excellent. Reaching for a browser, or for
matplotlib, is then a choice, not a necessity.

The goal is not the biggest chart catalog; it is the smallest vocabulary that
composes into one. Better a few strict rules than lots of features. One black
square on a plain ground beats a mural.

That means three commitments:

- **Eight marks, and the catalog follows.** Line, Points, Bars, Area, Cells,
  Range, Rule, Text. That is the alphabet. Give them a statistics layer and
  shared scales, and the charts the catalog has names for are words you spell
  with them. A chart type is a preset: the short spelling of one of those
  words, the same drawing as writing the word out in full.
- **Every claim can be checked.** The true chart draws every point.
  Aggregation has to land on the same pixels. A preset has to print the same
  bytes as the plot it abbreviates. An advertised number has a bench behind
  it, and every chart in these docs is program output, verified in CI. A
  claim is one assert away from proof.
- **Every terminal gets an answer.** Output steps down a ladder: real pixels,
  then octants, quadrants, ASCII. Color fades from truecolor down to plain.
  It never fails, never probes where an escape is unsafe, and never owns the
  screen.

A chart library usually grows by accretion. One function per chart type. One
option per request. Malevich does not. The plot is written once, as data.
Everything after it is a stage reading the same spec:

- Construction stacks layers on shared scales. The plot carries no terminal,
  no thread, and no global. It is `Clone + Send + Sync` because there is
  nothing in it that could not be.
- Resolution unions the layers' domains, places ticks, and lays out furniture,
  per frame, at render time. Large layers reduce to the raster here,
  pixel-identically.
- Rasterization draws marks onto a subpixel surface, or onto real pixels
  where the terminal speaks them. The same mark code serves both fidelities.
- Encoding writes the surface as a `String`. Glyphs and SGR for a terminal.
  Spans for a notebook card. Rectangles and text runs for a host that draws
  with SVG. Cells for a TUI buffer. The terminal is a category, not a device.
  Every host that draws the cell grid is one, and the encoder is the only
  thing that differs.

Adoption may follow. It is never chased. No chart-type races, no config
surface for its own sake, no dependency bazaar.

## The rules

Five rules, one axis each. They say what a chart means, what the core owns,
what is true, where that has to hold, and how a claim earns its place.

1. **The plot is the spec.** A plot is layers, scales, and furniture. That is
   data, complete and serializable. Rendering is a pure function of plot and
   frame. The frame is run state, not plot state, so one spec renders
   concurrently at many sizes. Environment reading lives only in named
   conveniences, never inside render.
2. **The grammar is closed.** Eight marks, a statistics layer, four position
   scales. A feature must be a composition of existing concepts. A new concept
   must pay for itself across many features. Every preset is provably its own
   grammar expansion, so the front door never forks the grammar.
3. **The full draw is the truth.** The oracle is drawing every point.
   Aggregation reproduces its pixels exactly. Extremes survive. `NaN` is a
   visible gap. Out-of-range data clips rather than smears. Quantization is
   disclosed. Nothing is sampled away silently.
4. **Every terminal is answered.** Rendering degrades down declared ladders,
   and it never fails. Furniture sheds before data. ASCII always works. Piped
   output is clean plain text. No escape byte is written where it is not
   safe. The library never owns the terminal.
5. **Claims are measured; figures are output.** Every advertised number comes
   from the bench suite and is recorded with its machine and date. Every chart
   in the documentation is spliced program output. It is regenerated, diffed,
   and failed in CI when stale. No figure is typed by hand.

## Principles

Constraints the vision names, without arguing them on this page. One file per
principle. The type names in each "Spelled today" section may rot. The rest
must not.

- [Presets are packaging](principles/presets-are-packaging.md)
- [The frame is run state](principles/frame-is-run-state.md)
- [The full draw is the oracle](principles/full-draw-oracle.md)
- [What earns a concept](principles/what-earns-a-concept.md)
- [Degradation is the contract](principles/degradation-is-the-contract.md)
- [The axes are the product](principles/axes-are-the-product.md)
- [Conversion lives at the rim](principles/conversion-at-the-rim.md)

## The name

Kazimir Malevich painted a black square on a plain ground and meant it: a
small vocabulary of geometric forms, composed deliberately. That is the design
budget of this library.
