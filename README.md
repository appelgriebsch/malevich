# malevich

Terminal plotting for Rust: a small grammar of marks, honest axes, millions of points.

**Status: early and moving fast (0.2).** Three marks — lines, points, bars — with real
axes: extended-Wilkinson tick placement with exact-decimal labels, legends, a zero-
baseline band scale for bars, function sampling at raster resolution, honest gaps, and
graceful degradation in small frames. Rendering spans four charsets (braille,
quadrants, half-blocks, ASCII) and four color tiers (truecolor, 256, 16, plain) with
honest downhill quantization; output is plain text automatically when piped. Breaking
changes between releases are expected until 1.0.

```rust
println!("{}", malevich::line(&[1.0, 5.0, 2.0, 8.0][..]));
```

<!-- generated:readme_sample -->
```text
8 ┤                         ⡠⠊
  │                       ⡠⠊
  │       ⣀⠤⠒⠤⣀        ⢀⠔⠉
4 ┤    ⡠⠔⠊     ⠉⠒⠤⣀  ⢀⠔⠁
  │⢀⡠⠔⠉            ⠉⠒⠁
0 ┤⠁
  └┬────────┬───────┬────────┬
   0        1       2        3
```
<!-- /generated -->

```rust
println!("{}", malevich::bar(["mon", "tue", "wed", "thu", "fri"], &[3.0, 7.0, 4.5, 8.0, 6.0][..]));
```

<!-- generated:readme_bars -->
```text
8 ┤         ▁▁▁▁▁         █████
  │         █████         █████  ▃▃▃▃▃
  │         █████         █████  █████
4 ┤         █████  █████  █████  █████
  │  ▆▆▆▆▆  █████  █████  █████  █████
  │  █████  █████  █████  █████  █████
0 ┤  █████  █████  █████  █████  █████
  └─────────────────────────────────────
      mon    tue    wed    thu    fri
```
<!-- /generated -->

Every chart in these docs is real program output, spliced in by
`cargo run --example regen_docs` and verified in CI — never typed by hand. More in the
gallery: [EXAMPLES.md](EXAMPLES.md), and `cargo run --example showcase` renders a
colored tour sized to your terminal.

## What it will be

- **A small closed vocabulary** — a handful of marks, stats, and scales that compose
  into the whole basic chart catalog (line, scatter, bars, histogram, heatmap, box,
  violin, ecdf, contour, …). Chart types are presets over the grammar, not separate
  implementations.
- **Axes that are actually good**: extended-Wilkinson tick placement, log and time
  scales, SI-prefix label formatting.
- **Millions of points**: data is aggregated to the known character raster in one fused
  pass (M4 min/max aggregation — pixel-exact for line rendering) before a single glyph
  is chosen. Measured: ten million points render end to end in about 28 ms,
  single-threaded (`cargo bench --bench render`).
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

## Acknowledgements

malevich stands on the shoulders of giants — the algorithms, libraries, and grammars
that taught this project what it knows are credited, specifically, in
[ACKNOWLEDGEMENTS.md](ACKNOWLEDGEMENTS.md).

## License

MIT OR Apache-2.0.
