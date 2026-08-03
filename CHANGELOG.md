# Changelog

Notable changes, written for humans. Pre-1.0, breaking changes are expected and listed
without apology.

## Unreleased

- `Plot::render_pixels_at(frame, graphics, column)`: pixel output anchored at
  a cell column — every text row and the image cursor walk start with an
  absolute-column jump (CHA; rows stay relative, so scrollback is safe),
  letting hosts paste a pixel plot beside other content. The showcase uses it:
  with the `pixel` feature in a capable terminal, every chart in
  `cargo run --example showcase --features pixel` renders as a side-by-side
  comparison — cells on the left, the same plot as a real image on the right.
- `pixel::Capabilities`: terminal capabilities as a plain queryable value —
  the protocols the terminal accepts (best first), its cell size in device
  pixels, and whether the answer was `Probed` or `Sniffed`.
  `Capabilities::detect()` now actively probes the terminal where that is
  safe (a real tty, no tmux/screen, `TERM` not dumb): one raw-mode
  `/dev/tty` round trip carrying the kitty graphics query, XTVERSION,
  XTSMGRAPHICS, and `CSI 16 t`, with DA1 as the ordering barrier — ground
  truth that, unlike `TERM_PROGRAM` sniffing, survives ssh. The probe runs
  at most once per process (~100 ms on answering terminals, 300 ms budget
  otherwise), an unanswered probe degrades to the sniff answer, and
  `Graphics::detect()` is now sugar for `Capabilities::detect().best()` —
  so `render_best` and the examples pick up probing for free. Try
  `cargo run --example pixels --features pixel -- --capabilities`.
- `Plot::render_best(&frame)`: renders at the best graphics tier the terminal
  offers — the plot panel becomes a real image when the `pixel` feature is on
  and a protocol is detected, and is exactly `render(&frame)` everywhere else
  (pipes, unknown terminals, tmux, or without the feature). The gallery
  examples now use it, so `cargo run --example sine --features pixel` (or any
  other example) upgrades to pixels in a capable terminal while the
  deterministic gallery output stays byte-identical. `Display` is unchanged:
  `println!("{plot}")` stays cells-only.
- Pixel graphics (new feature `pixel`): `Plot::render_pixels` renders the plot
  panel as a real image — sixel, kitty graphics, or iTerm2 inline PNG — while
  title, axes, tick labels, and legend stay text cells. Marks draw at
  device-pixel resolution through the same generic pipeline (`render::Canvas`,
  new): M4 buckets per pixel column, heatmap cells sample per pixel, bars fill
  exact rectangles, box-plot medians read as cleared gaps, and in-panel `Text`
  marks blit a baked public-domain 8×8 font. Undrawn panel area is transparent
  (sixel `P2=1`, kitty alpha, PNG alpha) so the terminal background shows
  through; output remains a deterministic `String` woven with DECSC/DECRC
  relative cursor moves. `pixel::Graphics::detect()` sniffs the terminal's best
  protocol (kitty/ghostty → kitty; iTerm2/WezTerm → iTerm2; foot, Konsole ≥
  22.04, Windows Terminal → sixel; tmux, pipes, unknown → `None`) and reads the
  cell size from `TIOCGWINSZ`. All three encoders are hand-rolled — including
  the stored-deflate PNG with its checksums — adding zero required
  dependencies (`rustix`, already in-tree, joins as an optional dep for the
  cell-size ioctl). Try `cargo run --example pixels --features pixel`.

- Second demo app: `sysmon`, a live system monitor (`cargo run -p sysmon`) — a
  sampler thread streams CPU, memory, and network readings through
  `stream::Ring` sliding windows (network counters via `stream::Rate`) into a
  dashboard of pinned-axis area charts, an SI-prefixed bytes/s network chart, and
  a per-core utilization heatmap with colorbar. Demos now live in per-app crates
  (`demos/fred`, `demos/sysmon`).
- New demo app (`demos/`, a separate unpublished workspace member): `fred`, a Federal
  Reserve economic-data browser in ratatui with five views — small-multiples overview,
  a series view (line/step/corners styles, calendar axis, log and year-over-year
  transforms, NBER recession ribbon, a 2% target rule on inflation), change histograms
  with decade box plots, a month-by-year seasonality heatmap with colorbar, and the
  Phillips-curve scatter plus the 10y-minus-fed-funds spread. Pure data and view
  layers (unit-tested) under a thin TUI shell; live refresh from FRED; heavier deps
  stay out of the malevich crate and CI. Run: `cargo run -p malevich-demos --bin fred`.
- New gallery entry `charsets`: the same curve rendered across the whole charset
  ladder — octants, sextants, quadrants, half blocks, braille, ASCII — so the
  subpixel-density trade-off is finally visible in the docs, not just described.
- Grid (side-by-side plots) now leaves a blank row between stacked rows, matching the
  blank column already between neighbors. A lower row's title no longer butts against
  the row above's axis labels — multi-row small multiples read as distinct plots.

## 1.11.1 — 2026-08-02

- Declared MSRV: Rust 1.88 (`rust-version` in `Cargo.toml`), verified by a pinned
  CI job — the crate's let-chains and edition 2024 set the floor.
- Stability guardrails in CI now that the crate is 1.x: `cargo-semver-checks` compares
  the public API against the last published release (a break requires a major bump),
  and `cargo-deny` (see `deny.toml`) scans dependency advisories, licenses, and sources.

## 1.11.0 (White on White) — 2026-08-02

Crossing into 1.x. The version lineage is kept (major bumped, minor/patch as they
were) rather than reset — this is the same crate, matured, not a rewrite. The API is
what the Polish sweep settled; semver discipline begins here, so breaking changes now
mean a 2.0. (The remaining 1.0-hygiene items — a declared MSRV, `cargo-semver-checks`
and advisory scanning in CI — are tracked as follow-ups, not blockers.)

- Colorbars: `Plot::colorbar()` draws the colormap as a labeled strip down the right
  edge, legending a `Cells` layer's value range. The `heatmap` and `hist2d` presets
  turn it on by default (a color-coded grid with no value scale is half a chart); the
  bare `Cells` grammar stays uncolored-legend for full control. Sheds on narrow frames.

## 0.11.0 (Polish) — 2026-08-02

The API review before the 1.0 freeze: the breaking changes are settled here, while
the crate is still pre-1.0 and cheap to move on. A fallible boundary makes external
specs safe; M4's headline guarantee is real again; a few names stop lying.

- `Scale::Auto` is the new default, distinct from `Scale::Linear`. An automatic axis
  adapts to its layers (categorical when a bars or band-range layer is present,
  linear otherwise); an *explicitly* chosen scale is now always honored rather than
  silently overridden by a categorical layer. `Plot::validate` rejects a categorical
  layer under a numeric x scale, and categorical layers that disagree on their bands.
- Renames (breaking, landed early so downstream churn is minimal):
  - `stat::Grid` → `stat::Histogram2d` — it was a second public `Grid`, unrelated to
    the small-multiples `Grid` at the crate root; the name now says what it is.
  - `Ticks::step()` returns `Option<f64>` instead of `f64` — `None` for a lone tick or
    the non-uniform ticks of a log/time axis, rather than a `0.0` sentinel a caller
    could mistake for a real spacing.
- M4 is pixel-exact again — and honestly so. Large lines are now reduced in *mapped
  raster space*: a cheap min/max probe fixes the layout, then M4 buckets by the exact
  column each point renders into, so the downsampled raster is bit-identical to
  drawing every point (verified against a raw-render oracle across index and xy lines
  at several sizes). The extra probe pass trades a little speed — ~45 ms for ten
  million points, up from ~28 — for the restored guarantee; a single-pass path is
  tracked for later.
- A fallible validation boundary: `Plot::validate` checks a spec's invariants
  (paired channel lengths, rectangular grids, valid colormaps, finite manual
  domains, scale/domain compatibility) and returns the first problem as a typed
  `Error`; `Plot::try_render` validates then renders. `render` stays infallible and
  lenient — this is the strict counterpart for deserialized or configured specs.

## 0.10.1

Correctness hardening from an external audit. Most fixes make existing guarantees
real under composition, deserialization, and extreme inputs.

- Fixed domains (`x_domain`/`y_domain`) are now honored exactly — they no longer
  widen to the tick range — and every mark is clipped to the plot rectangle, so
  out-of-range data can no longer leak ink into the axes or a neighboring grid cell.
- Off-screen bar and area spans are clamped before rasterizing, so distant finite
  data under a narrow domain can no longer spin a near-unbounded draw loop.
- `Bins::auto` always covers the data and respects its cap: it widens the bin
  instead of dropping observations, so counts sum to the finite input count.
- `Moments::default()` now equals `Moments::new()` (extrema start unset, not `0`).
- M4 preserves a gap that falls inside a raster column — a `NaN` between two values
  no longer reconnects them. Downsampling is described honestly as silhouette-
  preserving; true pixel-exactness (mapped-space bucketing) is tracked for later.
- Deserialized specs that violate constructor invariants (empty colormap,
  zero-column grid, ragged range/area channels) now render defensively instead of
  panicking.
- `Ticks::linear` no longer panics or hangs on extreme finite bounds; `kde` declines
  a degenerate large-magnitude sample instead of over-allocating; `hist2d` of
  constant data renders instead of coming out blank; a log axis with a non-positive
  manual domain is clamped rather than panicking, and a value that maps off a log
  axis is treated as a gap.
- `lttb` and `m4` assert equal-length inputs, like the mark constructors; the
  `contour` preset validates its geometry and treats all non-finite values as gaps.
- Range body values now participate in y-axis fitting, so a body reaching past the
  whiskers is no longer clipped.

## 0.10.0 (Reach)

- Contour lines: `stat::contours` (marching squares — canonical shared-edge
  interpolation, center-average saddles, NaN gaps) and the `contour` preset with
  tick-chosen levels, colormap-graded and legend-labeled.
- `quiver` preset: a vector field as arrows drawn in data coordinates.
- `serde` feature: every spec type round-trips (plots, marks, scales, themes,
  frames, grids). Series gaps encode as `null` in JSON and decode back to gaps;
  function-backed lines refuse to serialize honestly.
- `ndarray` feature: one-dimensional arrays and views ingest directly, zero-copy
  when contiguous.
- `Colormap` stops are copy-on-write (`Colormap::new` is still const); `Colormap`
  is no longer `Copy`.

Deliberately not added: a pie preset (no x/y scales — it fights the marks-over-scales
grammar; part-to-whole is served by `bar`) and a `polars` dependency (too large;
polars already reaches a chart with no dependency through the zero-copy slice path —
see the README).

## 0.9.0 (Red Cavalry) — 2026-08-02

Riding into the ratatui ecosystem.

- The ratatui adapter (feature `ratatui`, depending only on `ratatui-core`):
  `plot.widget()` renders any chart straight into a `Buffer` — no ANSI round-trip,
  colors map onto cell styles, the host application keeps the terminal. Charset and
  theme are widget options; `cargo run --example tui --features ratatui` shows a
  live dashboard.
- The gallery now runs on real data (`examples/data/`, with provenance and
  licenses): the Keeling curve (NOAA, public domain), Palmer penguins (CC0), and a
  genuinely real training log — 1,000 per-step losses captured from poorgrad's
  bigram model. Six entries converted; mathematical examples stay mathematical.
- The corners line style (`LineStyle::Corners`): the classic asciichart look —
  one box-drawing glyph per column, `╭╮╰╯` elbows, `│` runs — with real axes
  underneath, and an honest `+`/`-`/`|` fallback in ASCII charsets.
- Retained-plot cloning measured at ~10 µs for 12 layers × 5k points
  (`plot/clone_12x5k_owned`) — cheap enough that no copy-on-write machinery is
  warranted.

## 0.8.0 (Black Cross) — 2026-08-02

The layout release: the charset ladder completes, and plots compose into grids.

- Sextant (2×3, Unicode 13) and octant (2×4, Unicode 16) charsets: braille density
  with solid ink. `Frame::detect` now auto-selects octants on terminals known to
  render them (kitty, ghostty, WezTerm, foot, recent VTE, Windows Terminal) —
  sniffed, never probed.
- Small multiples (`Grid`): plots pasted side by side with escape-aware padding;
  share axes by fixing domains, not by a mode.
- Manual axis domains (`Plot::x_domain`, `Plot::y_domain`): matplotlib's xlim/ylim;
  data outside clips honestly.

## 0.7.0 (Eight Red Rectangles) — 2026-08-02

The quality release: typed scales, named axes, and honest ASCII — driven by the
first full audit.

- The scale specification (`Scale`: `Linear | Log | Time | Bands`, via
  `Plot::x_scale`/`y_scale`): one typed axis spec replaces the three boolean flags
  (which remain as sugar); an explicit `Scale::Bands` declares a categorical axis
  without needing a bar layer — the violin preset now uses it instead of a
  data-free range.
- Axis titles (`Plot::x_label`, `Plot::y_label`): x centered under the tick labels,
  y written vertically along the left edge; both shed when the frame is tight.
- Internal: the plot pipeline split into stage modules (resolve → layout → chrome →
  draw) — verified byte-identical by the golden suite; crate-level rustdoc rewritten
  (it had been six releases stale).

## 0.6.0 (The Knife Grinder) — 2026-08-02

Time and motion: calendar axes, rolling windows, and live charts.

- Time axes (`Plot::time_x`, `Ticks::time`): unix seconds in, calendars out — a
  1s-to-decades interval ladder aligned to real boundaries (Mondays, month firsts),
  multi-scale labels (`14:05`, but `Aug 2` at midnight and `2027` at January), exact
  Gregorian arithmetic, UTC, no dependencies.
- Rolling windows (`stat::Window`): trailing mean/sum/min/max with partial starts
  (no warm-up gap) and gap-aware reductions.
- Streaming (`stream::Ring`, `stream::Live`, `stream::Rate`): a thread-shared
  sliding window (the library's one lock — producers push, renderers snapshot),
  an in-place repaint handle (cursor up, erase down, one buffered write:
  flicker-free, scrollback-safe), and a counter-to-delta helper. One live frame
  renders in well under a millisecond (see `benches/render.rs`).

## 0.5.0 (Sportsmen) — 2026-08-01

The statistics release: the mark family is complete, and the statistical charts no
terminal library ships are here.

- Range (`mark::Range`): the eighth and final mark — vertical intervals with
  optional `body` and `marker` channels, so error bars, boxes, and candles are one
  mark with channels, not three marks. Band placement (`Range::over`) shares the
  categorical axis machinery with bars.
- Box plots (`stat::BoxStats`, `malevich::box_plot`): type-7 quartiles, Tukey 1.5×IQR
  whiskers, outliers as dots.
- Densities (`stat::kde`, `malevich::density`, `malevich::violin`): Gaussian KDE with
  Silverman bandwidth over linear binning (no FFT); violins as mirrored densities via
  the new horizontal area orientation (`Area::horizontal`).
- Error bars (`malevich::error_bars`): capped Range intervals around measured points.

## 0.4.0 (Suprematist Composition) — 2026-08-01

The daily driver: the mark family grows to seven, and the statistical presets with it.

- Cells (`mark::Cells`, `scale::Colormap`, `malevich::heatmap`, `malevich::hist2d`,
  `stat::bins2`): value grids as a shade ramp (`░▒▓█`) colored by a colormap
  (viridis-like default) — value carried by glyph and color, readable at every tier
  including plain; grids map onto data coordinates via `Cells::extents`; empty 2D
  bins stay honestly blank.

- Area (`mark::Area`): baseline fills and between-bands, drawn as vertical subpixel
  runs — solid in every charset, subpixel edge precision, gap-breaking. `stat::stack`
  turns series into cumulative bands for stacked areas.
- Annotations (`mark::Rule`, `mark::Text`): reference lines and notes at data
  coordinates; both extend the axis domains, draw in the default foreground, and
  never consume palette slots.
- Steps (`malevich::stairs`, `malevich::ecdf`, `stat::ecdf`): step charts and
  empirical distributions as presets over the line mark.

## 0.3.0 (Airplane Flying) — 2026-08-01

The pipeline release: the stat layer lands, and ten million points become cheap.

- Histograms (`stat::Bins`, `mark::Bars::spans`, `malevich::hist`): automatic bin
  counts (Sturges/Freedman–Diaconis) with nice decimal edges, mergeable bin counts,
  and contiguous span bars on a numeric axis.
- Group-by (`stat::Agg`): string-keyed grouping with the shared reducer vocabulary —
  `count`, `sum`, `mean`, `min`, `max`, `median` — feeding `Bars::new` directly.
- Log axes (`Plot::log_x`, `Plot::log_y`, `Ticks::log10`): decade ticks with
  superscript labels; values at or below zero become gaps, because a log axis cannot
  place them honestly.
- The aggregation pipeline (`stat`): M4 downsampling (`stat::M4`, `stat::m4`) —
  min/max/first/last per raster column, pixel-exact for line rendering, mergeable
  across chunks, gap-preserving — inserted automatically for line layers past four
  points per subpixel column. Ten million points render end to end in ~28 ms
  (measured; see `benches/render.rs`). Also `stat::lttb` (count-targeted,
  shape-preserving) and `stat::Moments` (Welford + Chan merge).
- SI-prefixed tick labels: axes reaching ±10⁴ (or below 10⁻³) share one prefix
  (`20k`, `2.5M`, `100µ`); the numeric part times the prefix still equals the value
  exactly, and zero stays bare.

## 0.2.0 (Red Square) — 2026-08-01

Color and the next two marks: the chart, the dots, and the bars now look considered
at every color tier.

- Half-block (`▀▄█`, 1×2) and quadrant (`▘▚▟`…, 2×2) charsets: solid-block
  alternatives to braille, selectable per frame.
- Legends: `.label("…")` on any mark grows a legend row with per-kind colored
  swatches, shed first when the frame is short.
- Themes (`Theme`, `Frame::theme`): the palette as a value — `DARK` (default),
  `LIGHT` (readable on white), `COLORFGBG` detection, or any custom palette.
- Bars (`mark::Bars`, `scale::Band`, `malevich::bar`): categorical bar charts from a
  zero baseline with eighth-block partial tops, coarse below-baseline fills for
  negative values, band-fitted category labels, and continuous layers (trend lines)
  positioning over band centers.
- Points (`mark::Points`, `malevich::scatter`): unconnected dots; marks now join
  under the closed `mark::Mark` enum and `Plot::layer(impl Into<Mark>)`.
- Color ladder (`Color::{Ansi256, Rgb}`, `ColorMode::{Plain, Ansi16, Ansi256,
  TrueColor}`): honest downhill quantization (RGB → 256-cube → nearest-16), named
  colors stay palette-relative, run-length encoding merges colors that quantize
  equal. Detection adds `CLICOLOR_FORCE`, `COLORTERM`, `256color` terms, `TERM=dumb`,
  and non-UTF-8 locale sniffing.
- Display-width discipline: labels measured in terminal columns (CJK-safe), wide
  glyphs pair with continuation cells and never corrupt alignment, truncation uses an
  ellipsis. New dependency: `unicode-width`.

## 0.1.0 (Black Square) — 2026-08-01

The vertical spine: one mark, every layer of the architecture, done properly.

- The plot pipeline (`Plot`, `Frame`, `mark::Line`, `malevich::line`): layered line
  charts over shared scales with measured (never fixed) layout, collision-aware x
  labels, chrome shedding in undersized frames, function sampling at raster
  resolution, a default palette, and `Display` via `Frame::detect`. Presets are
  asserted bit-identical to their grammar expansion.
- The examples gallery (`EXAMPLES.md` + `regen_gallery`): deterministic, CI-checked —
  the showcase and the system test in one artifact.
- Rendering (`render::Surface`, `render::Charset`, `render::Color`): one generic
  subpixel surface over charset codecs (braille 2×4 and ASCII for now), clipped
  infallible drawing, text sharing the grid with pixels, plain and run-length ANSI
  encoders.
- Data ingestion (`data::Series`, `data::IntoSeries`): zero-copy from `f64` slices,
  copy-once conversion from all primitive numeric types, `NaN` preserved as the gap
  encoding.
- Tick placement (`scale::Ticks`): extended Wilkinson (Talbot–Lin–Hanrahan) with
  exact-decimal labels — labels parse back to their values, share one fraction width
  per axis, and never show float artifacts. Placement runs in microseconds.
- Project scaffold: crate skeleton, terminology contract, CI.
