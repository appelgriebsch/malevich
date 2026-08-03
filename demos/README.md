# malevich demos

Flagship terminal apps built on [malevich](https://crates.io/crates/malevich) — real
tools, not snippets, showing the library driving live [ratatui](https://ratatui.rs)
dashboards. Each app is its own unpublished workspace member, so their heavier
dependencies (ratatui, ureq, sysinfo) never touch the malevich crate or its CI.

Every demo splits three ways, mirroring malevich's own spec-then-render philosophy:
a pure data layer (unit-tested), a pure view layer (data in, `Plot` out), and a thin
binary that owns the terminal. The same view functions power the TUI and each app's
headless `--render` mode, and the view code carries comments explaining each
malevich concept it uses.

## [fred](fred/) — a Federal Reserve economic-data browser

```
cargo run -p fred
```

Five views over US economic data: a small-multiples overview, a series view
(line/step/corners styles, calendar axis, log and year-over-year transforms, NBER
recession shading, a 2% target rule on inflation), change histograms with decade box
plots, a month-by-year seasonality heatmap with colorbar, and the Phillips-curve
scatter plus the 10y − fed-funds spread. Vendored public-domain data; `f` refreshes
live from FRED.

## [sysmon](sysmon/) — a live system monitor

```
cargo run -p sysmon
```

The streaming story: a sampler thread pushes CPU, memory, and network readings into
`malevich::stream::Ring` sliding windows (network counters through `stream::Rate`);
the UI snapshots and redraws four times a second. A dashboard of filled area charts
(CPU pinned to 0–100, memory to the machine total, network on an SI-prefixed
bytes/s axis) and a per-core utilization heatmap with colorbar over instantaneous
load bars.
