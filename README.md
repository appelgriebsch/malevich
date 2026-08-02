# malevich

Terminal plotting for Rust: a small grammar of marks, honest axes, millions of points.

**Status: pre-0.1, under construction.** Nothing usable yet. The first release, *Black
Square*, ships one thing done properly: a line chart with real axes — principled tick
placement, braille and ASCII rendering, honest gaps, plain output when piped.

## What it will be

- **A small closed vocabulary** — a handful of marks, stats, and scales that compose
  into the whole basic chart catalog (line, scatter, bars, histogram, heatmap, box,
  violin, ecdf, contour, …). Chart types are presets over the grammar, not separate
  implementations.
- **Axes that are actually good**: extended-Wilkinson tick placement, log and time
  scales, SI-prefix label formatting.
- **Millions of points**: data is aggregated to the known character raster in one fused
  pass (M4 min/max aggregation — pixel-exact for line rendering) before a single glyph
  is chosen.
- **Renders to cells, never owns the terminal**: a `String` for CLIs, logs, and CI; a
  ratatui widget for TUIs; plain text automatically when piped.
- **A capability ladder**: ASCII → blocks → braille → Unicode 16 octants; 16 → 256 →
  truecolor; `NO_COLOR` respected; graceful degradation when space or color runs out.
- **Plots are plain values**: no global state, `Send + Sync` throughout, rendering is a
  pure function — build a plot on one thread, render it on another.

## What it will not be

Not a TUI framework (it never owns the terminal or handles input). No animations. No
file parsing or dataframes in core — ingestion traits only. No config-object kitchen
sink.

## Name

Kazimir Malevich painted a black square on a plain ground and meant it: a small
vocabulary of geometric forms, composed deliberately. That is the design budget of this
library.

## License

MIT OR Apache-2.0.
