# Performance

Fast is a feature, and claims are measured. Every advertised number has a
bench behind it, recorded with its machine and date. This file is the
public story. [BENCHMARKS.md](../BENCHMARKS.md) is the authoritative dated
record it summarizes.

Speed and honesty coexist because of the oracle. The fast path is proven
byte-identical to drawing every point, so there is no fidelity knob to trade
away. See
[The full draw is the oracle](principles/full-draw-oracle.md).

## The mechanisms

- **Lines: M4 to the raster.** Large line layers reduce to
  min/max/first/last per raster column, bucketed by the column each point
  renders into. O(n) once, then O(width × height), and pixel-identical to
  the full draw by construction. The reduction is auto-inserted past four
  points per column.
- **Grids: bucket-exact reduction.** Cells grids denser than the raster
  reduce through the shared `Reducer` vocabulary. Every screen bucket owns
  the cells whose centers fall inside it. Cost is linear in the grid, about
  10 ns per cell, not in the raster.
- **Geometry without a raster.** `Plot::mapping`, what an interactive host
  calls between renders, runs the extent probe and the layout pass only.
  The aggregation is never computed for a raster nobody draws.
- **No allocation per point.** Rendering retains labels and identities at
  construction and borrows them after that. CI enforces allocation ceilings
  (at most 275 allocations and 64 KiB for the 10k-point render), so a
  structural regression — an allocation per point, a new large intermediate —
  fails the build.

## Measured

Two machines, single-threaded, end to end: construct, resolve, reduce,
rasterize, encode, at 1.23.0. Order of magnitude and which mechanism wins,
not a promise for your machine. The same code runs 1× to 20× slower on the
virtualized Xeon than on the laptop, and not uniformly.

| measurement | Apple M1 Pro | x86_64 Xeon (KVM) |
|---|---:|---:|
| line, 10,000 points, 80×20 | 69 µs | 152 µs |
| line, 10,000,000 points, 80×20 | 31 ms | 142 ms |
| cells, 2048×2048 grid onto 80×24 | 41 ms | 42 ms |
| geometry only (`Plot::mapping`), 10,000,000 points | 2.1 ms | 46 ms |
| streaming least squares, 1M pairs (`stat::Fit`) | 5.1 ms | 6.4 ms |
| 100,000 points, 5 categories via `color_by` | 2.0 ms | 3.4 ms |
| 100,000 points, 100,000 categories | 5.3 ms | 8.0 ms |
| one interactive frame, 10,000,000 points at a 1% zoom, 200×50 | 18 ms | 166 ms |
| the same frame with a hover cursor | 25 ms | 199 ms |
| two-pane dashboard, 200×50 (100k-point lines beside a 256×128 heatmap) | 1.8 ms | 4.1 ms |

The categorical pair is a structural fence. Runtime grows with input plus
legend size, not input × category count. The ten-million-point row is the
README's "tens of milliseconds" claim, on the M1 Pro record it cites. The
cells row is its matrix analog, 4.19 million cells onto ~4k screen buckets,
and the one row the two machines agree on. The interactive rows are what a
TUI pays per frame while someone pans or zooms.

The Xeon column is the first record on a second architecture, and it says
something the laptop alone could not. The per-point walks are where the
machines diverge most: M4, the extent probe, the hover cursor's nearest
scan. The per-bucket and per-cell work costs about the same on both. On the
Xeon the M4 reduction alone (`stat/m4_10m_160cols`) is 127 of the 142 ms.
The record states those ratios. It does not explain them.

## Rerun on yours

```sh
cargo bench --bench render -- render/line_10m_80x20
cargo bench --bench render -- render/cells_2048x2048_80x24
cargo bench --bench alloc
cargo run --example bench_record       # the saved results as a record block
```

The bench suite is the only source docs quote numbers from.
`bench_record` prints its saved results in the record's format: Criterion's
own estimates and intervals, with the machine, OS, compiler, and revision
they came from. No number in the record is typed by hand. Baselines,
machine details, confidence intervals, and the update protocol live in
[BENCHMARKS.md](../BENCHMARKS.md).
