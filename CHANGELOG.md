# Changelog

Notable changes, written for humans. Pre-1.0, breaking changes are expected and listed
without apology.

## 0.9.0 (Red Cavalry) — 2026-08-02

Riding into the ratatui ecosystem.

- The ratatui adapter (feature `ratatui`, depending only on `ratatui-core`):
  `plot.widget()` renders any chart straight into a `Buffer` — no ANSI round-trip,
  colors map onto cell styles, the host application keeps the terminal. Charset and
  theme are widget options; `cargo run --example tui --features ratatui` shows a
  live dashboard.
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
