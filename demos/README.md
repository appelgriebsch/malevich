# malevich demos

Flagship terminal apps built on [malevich](https://crates.io/crates/malevich) — real
tools, not snippets, showing the library driving a live [ratatui](https://ratatui.rs)
dashboard. This is a separate, unpublished workspace member, so its heavier
dependencies (ratatui, ureq) never touch the malevich crate or its CI.

Each demo splits three ways, mirroring malevich's own spec-then-render philosophy:
a pure data layer (`src/fred/data.rs` — parsing, calendar math, transforms;
unit-tested), a pure view layer (`src/fred/views.rs` — data in, `Plot` out), and a
thin binary that owns the terminal. The same view functions power the TUI and the
headless `--render` mode.

## fred — a Federal Reserve economic-data browser

```
cargo run -p malevich-demos --bin fred
```

Five views over US economic data (unemployment, CPI, real GDP, fed funds, the
10-year yield, nonfarm payrolls), each a different corner of malevich's catalog:

| View | What it shows | Charts |
|---|---|---|
| **overview** | every series at a glance | six small-multiple line charts |
| **series** | one series large, with transforms | line / step / corners styles, calendar axis, log scale, YoY, NBER recession ribbon, a 2% target rule on inflation |
| **distribution** | how the series' changes distribute | histogram of period changes + level box plots by decade |
| **seasonality** | period changes by month and year | heatmap with a colorbar (the 2021–22 inflation is one hot band) |
| **relations** | cross-series classics | the Phillips-curve scatter split at 2000, and the 10y − fed-funds spread with its inversion rule |

Keys:

- `1–5`, `Tab`/`Shift-Tab`, `←/→` — switch view
- `↑ ↓` / `j k` — pick a series
- `t` — cycle the series transform: level → year-over-year → log axis
- `c` — cycle the line style (braille pixels ↔ asciichart corners)
- `g` — cycle the glyph charset (braille → octants → quadrants → half blocks)
- `s` — toggle recession shading
- `f` — refresh the selected series **live from FRED** (open CSV endpoint, no key)
- `q` — quit

It ships with a vendored snapshot (see [`data/README.md`](data/README.md)) so it runs
offline. Headless mode prints any view and exits — handy for piping or screenshots:

```
cargo run -p malevich-demos --bin fred -- --render seasonality CPIAUCSL
cargo run -p malevich-demos --bin fred -- --render relations
```
