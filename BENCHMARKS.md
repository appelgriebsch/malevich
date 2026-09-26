# Benchmark baselines

Performance here is a measurement, not a speed you can promise on another
machine. Wall-clock time moves with the hardware, the compiler, the power
state, and whatever else is running. This file is the dated record behind
the README's “tens of milliseconds” claim.

## 2026-09-24 addition (released in 1.23.0)

- Revision: `af024e1` (the borrowing series, closed by the commit that
  converts colormap stops to OKLab once per raster)
- Machine, OS, profile: as in the 2026-08-07 baseline below
- Compiler: `rustc 1.100.0-nightly (787af2b8c 2026-08-25)`, as in the
  2026-08-28 additions

| Measurement | Estimate | 95% interval |
| --- | ---: | ---: |
| `render/line_10k_80x20` | 69.323 µs | 69.226–69.430 µs |
| `render/line_10m_80x20` | 30.714 ms | 30.685–30.742 ms |
| `stat/fit_1m` | 5.1339 ms | 5.1273–5.1409 ms |
| `render/color_by_100k/5_categories` | 2.0211 ms | 2.0176–2.0248 ms |
| `render/color_by_100k/100000_categories` | 5.3475 ms | 5.3369–5.3587 ms |
| `render/cells_2048x2048_80x24` | 40.555 ms | 40.500–40.610 ms |
| `plot/mapping_10m_80x20` | 2.0800 ms | 2.0681–2.1015 ms |
| `widget/dashboard_200x50` | 1.7717 ms | 1.7692–1.7742 ms |
| `widget/zoom_10m_200x50` | 18.366 ms | 18.350–18.383 ms |
| `widget/hover_snap_10m_200x50` | 25.368 ms | 25.138–25.789 ms |

```sh
cargo bench --bench render -- 'render/line_10k_80x20|render/line_10m_80x20|stat/fit_1m|render/color_by_100k|render/cells_2048x2048_80x24|plot/mapping_10m_80x20'
cargo bench --bench widget --features ratatui
```

The release changed the render path in several places. Bar ends map to
subpixel edges, colormaps sample through one `Colormap::sample`, colors mix
in OKLab, and axes search for context notes, so every prior row was
measured again. The line, fit, category, mapping, zoom, and hover rows sit
within noise of their 2026-08-28 values. Two rows moved. The cells row is
8% lower than its 2026-08-25 value, because the sampling closure no longer
converts a colormap's stops for every patch. The dashboard row is 32% higher
than its 2026-08-28 value, and that is the disclosed cost of perceptual
color. Every sampled heatmap cell now encodes its OKLab mix back to sRGB
(three gamma encodes per sample), where the RGB lerp was three integer
multiplies. An intermediate tree that also converted both neighboring stops
per sample measured 2.68 ms. Converting them once per raster is what the
closing commit does. The rows that do not touch a colormap were measured
one commit earlier, on a tree identical along their paths.

## 2026-08-28 addition — the mapping pass (released in 1.20.0)

- Revision: `3e2f63a` (the commit introducing `Plot::mapping`'s layout-only
  pass)
- Machine, OS, profile: as in the 2026-08-07 baseline below
- Compiler: `rustc 1.100.0-nightly (787af2b8c 2026-08-25)`, as in the
  addition below

| Measurement | Estimate | 95% interval |
| --- | ---: | ---: |
| `plot/mapping_10m_80x20` | 2.1059 ms | 2.0819–2.1389 ms |
| same row, previous full-preparation path | 30.425 ms | 30.394–30.457 ms |

```sh
cargo bench --bench render -- plot/mapping_10m_80x20
```

`Plot::mapping` is what an interactive host calls between renders. It now
runs only the extent-probe resolve and the layout pass. The mapped M4
aggregation it used to run was work for a raster nobody drew. The before
row was measured on the same tree, with `mapping` temporarily routed back
through the full render preparation. It matches the full-view render anchor
below within noise, which confirms the discarded work was the aggregation
itself.

## 2026-08-28 addition (released in 1.20.0)

- Revision: `a6f03ec` (the interactive-widget series)
- Machine, OS, profile: as in the 2026-08-07 baseline below
- Compiler: `rustc 1.100.0-nightly (787af2b8c 2026-08-25)`, newer than the
  baseline's stable. The anchor row was measured again on this compiler, so
  the new rows compare in place

| Measurement | Estimate | 95% interval |
| --- | ---: | ---: |
| `widget/dashboard_200x50` | 1.3376 ms | 1.3313–1.3483 ms |
| `widget/zoom_10m_200x50` | 18.606 ms | 18.567–18.645 ms |
| `widget/hover_snap_10m_200x50` | 25.015 ms | 24.949–25.083 ms |
| `render/line_10m_80x20` (anchor, this compiler) | 30.536 ms | 30.503–30.570 ms |

```sh
cargo bench --bench widget --features ratatui
```

What one interactive frame costs: rasterize, blit the buffer, and do not
encode a string. The dashboard row is a two-pane 200×50 frame through the
stateless widget, a legended two-series 100k-point line chart beside a
colorbarred 256×128 heatmap. The zoom row is the interactive one. A
ten-million-point line, rendered with state, at a fixed 1% window: that is
the cost of every frame while someone pans or zooms. M4 walks the full
series and re-aggregates into the window each time. Points outside the
window fail the column test early, which is why the zoomed frame undercuts
the full-view anchor. The hover row adds a cursor to that same state. The
snap readout's nearest scan over ten million explicit x values, plus the
overlays, comes to about 6.4 ms, roughly 0.6 ns per point, a linear scan at
memory bandwidth. The hovered frame stays at 40 fps. A line positioned by
index (`Line::y`) snaps in constant time and skips that cost entirely.

## 2026-08-25 addition (released in 1.18.0)

- Revision: `b0887bc` (the commit introducing the measured reduction)
- Machine, OS, compiler, profile: as in the 2026-08-07 baseline below

| Measurement | Estimate | 95% interval |
| --- | ---: | ---: |
| `render/cells_2048x2048_80x24` | 43.720 ms | 43.535–43.944 ms |

```sh
cargo bench --bench render -- render/cells_2048x2048_80x24
```

This is the matrix version of the ten-million-point line. 4.19 million cells
max-reduce onto ~4k screen buckets, in tens of milliseconds, about 10 ns per
cell. The bucket-exact reduction walks every covered cell once, so the cost
is linear in the grid, not in the raster.

## 2026-08-24 baseline (1.17.0)

- Revision: `7bbc202`
- Machine, OS, compiler, profile: as in the 2026-08-07 baseline below

| Measurement | Estimate | 95% interval |
| --- | ---: | ---: |
| `render/line_10k_80x20` | 69.182 µs | 68.837–69.594 µs |
| `render/line_10m_80x20` | 31.878 ms | 31.752–32.034 ms |
| `stat/fit_1m` | 5.2392 ms | 5.2163–5.2711 ms |
| `render/color_by_100k/5_categories` | 2.0587 ms | 2.0502–2.0685 ms |
| `render/color_by_100k/100000_categories` | 5.3453 ms | 5.3271–5.3665 ms |

```sh
cargo bench --bench render -- render/line_10k_80x20
cargo bench --bench render -- render/line_10m_80x20
cargo bench --bench render -- stat/fit_1m
cargo bench --bench render -- render/color_by_100k/5_categories
cargo bench --bench render -- render/color_by_100k/100000_categories
```

Against 1.16.0 on the same machine, the 10k render is 4.1% higher, after
making gaps explicit path topology. The ten-million-point render is 5.9%
lower, after selecting the ordinary affine map once and keeping each M4
bucket's current run directly addressable. The fit result is within 0.6% of
its prior estimate.

The categorical rows are the new fence. Both render the same 100,000 points.
The second also carries 100,000 distinct labels and identities. Runtime grows
with the input plus the legend, not with input × category count, which is
what the old masked-layer expansion did.

## 2026-08-15 baseline (1.16.0)

- Revision: `3b17b0d`
- Machine, OS, compiler, profile: as in the 2026-08-07 baseline below

| Measurement | Estimate | 95% interval |
| --- | ---: | ---: |
| `render/line_10k_80x20` | 66.466 µs | 66.322–66.615 µs |
| `render/line_10m_80x20` | 33.878 ms | 33.659–34.194 ms |
| `stat/fit_1m` | 5.2083 ms | 5.2019–5.2149 ms |

```sh
cargo bench --bench render -- render/line_10k_80x20
cargo bench --bench render -- render/line_10m_80x20
cargo bench --bench render -- stat/fit_1m
```

Rerun for 1.16.0 because resolution changed: the `color_by` layer expansion
and the shared line-reduction helper. The render rows came out 2.2% and 6.5%
lower than the 2026-08-07 baseline on the same machine. The categorical
channel costs the headline path nothing measurable.

`stat/fit_1m` is one million `(x, y)` pairs through the streaming
least-squares accumulator (`stat::Fit`). Bivariate Welford, one thread, and
no allocation in the loop. The accumulator merges associatively, so a host
can split this scan across chunks and combine them.

## 2026-08-07 baseline

- Revision: `7ff2bc0`
- Machine: 2021 MacBook Pro, Apple M1 Pro (10 cores), 32 GB RAM
- OS: macOS 26.5.2 (Darwin 25.5.0), arm64
- Compiler: `rustc 1.97.1 (8bab26f4f 2026-07-14)`, LLVM 22.1.8
- Profile: Cargo `bench` / optimized, Criterion 0.5 default 100-sample run

| Measurement | Estimate | 95% interval |
| --- | ---: | ---: |
| `render/line_10k_80x20` | 67.928 µs | 67.887–67.973 µs |
| `render/line_10m_80x20` | 36.226 ms | 36.202–36.251 ms |

Commands:

```sh
cargo bench --bench render -- render/line_10k_80x20
cargo bench --bench render -- render/line_10m_80x20
```

The benchmark is the whole path. Build the preset, resolve domains and
layout, run M4, rasterize an 80×20 braille frame, and encode the final
string. One thread. The ten-million-point input vectors are built before the
timer starts.

The earlier `0f3ad5a` record on this machine was 81.818 µs and 42.260 ms,
respectively. The current measurements are 17.0% and 14.3% lower. Same
machine, looking back. Not a promise about any other machine.

### Profiling decision

A five-second Instruments Time Profiler capture of the 10k case put 2,670
of 5,108 leaf samples (about 52%) on resolution. A measured A/B kept the
compact resolved-layer probe instead of copying every mark's domain rules
into a parallel metadata type. The smaller design was faster, and it keeps
one source of truth. The accepted change, instead:

- keeps implicit coordinates symbolic;
- summarizes a line into only the two endpoints its linear or log axis needs;
- keeps the probed layout for drawing, so tick formatting, gutter
  measurement, and colorbar work do not run a second time.

Cell rasterizers and hybrid device-pixel rasterizers both use that same
prepared-render phase. The target policy holds only sampling density, marker
cycling, downsampling, and the pixel fallback for corner glyphs that exist
only as cells. There is no parallel mark metadata.

The pixel-exact raw-versus-M4 oracle, and every rendering snapshot, stayed
identical.

## Allocation contract

At revision `7bbc202`, optimized on the machine above, the 10k render measured **183
allocations and 58,388 allocated bytes**, and wrote 2,966 output bytes. A 100k-point
plot with one unique category per point measured **67 allocations and 34,141 bytes**,
and wrote 1,791 output bytes. Rust 1.88 is what CI trusts for the ceilings:

```sh
cargo bench --bench alloc
```

CI runs that harness on Ubuntu 24.04 with Rust 1.88 and `--check`. It permits
at most 275 allocations and 64 KiB of heap traffic. The ceilings leave room
for compiler and allocator details, and they still catch a structural
regression: an allocation per input point, or a new large intermediate
buffer. CI does not gate wall-clock time on shared runners.

The line measurement includes gap-aware M4 state and the two-color cell
surface. The categorical measurement shows that rendering does not allocate
per point or per category. Labels and identities are kept when the mark is
built, and render preparation borrows them. The 64 KiB ceiling is unchanged,
and it still catches a larger per-cell representation or a manufactured
intermediate.

To update this record, benchmark a machine that is otherwise idle. Record the
revision, the hardware, the OS, the compiler, the commands, the point
estimates, and the confidence intervals. Change the allocation ceilings only
when a reviewed design change explains the new traffic.
