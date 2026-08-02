# Changelog

Notable changes, written for humans. Pre-1.0, breaking changes are expected and listed
without apology.

## Unreleased (toward 0.2.0)

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
