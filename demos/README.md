# malevich demos

Flagship terminal apps built on [malevich](https://crates.io/crates/malevich) — real
tools, not snippets, showing the library driving a live [ratatui](https://ratatui.rs)
dashboard. This is a separate, unpublished workspace member, so its heavier
dependencies (ratatui, ureq) never touch the malevich crate or its CI.

## fred — a Federal Reserve economic-data browser

```
cargo run -p malevich-demos --bin fred
```

Browse US economic series — unemployment, CPI, real GDP, the fed funds rate, the
10-year Treasury yield, nonfarm payrolls — as malevich line charts on a real calendar
time axis, with NBER recessions marked along the bottom.

- `↑ ↓` / `j k` — pick a series
- `t` — cycle the transform: **level → year-over-year % → level on a log axis**
- `s` — toggle recession shading
- `f` — refresh the selected series **live from FRED** (open CSV endpoint, no API key)
- `q` — quit

It ships with a vendored snapshot (see [`data/README.md`](data/README.md)) so it runs
offline; `f` pulls the latest. A headless mode prints one chart and exits — handy for
piping or a screenshot:

```
cargo run -p malevich-demos --bin fred -- --render UNRATE
```

Shows off: calendar time axes, log scales, year-over-year transforms, layered
`Area` shading behind a `Line`, and the ratatui `PlotWidget` embedding a chart in a
multi-panel TUI.
