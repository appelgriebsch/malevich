# Benchmark baselines

Performance here is a measurement, not a speed you can promise on another
machine. Wall-clock time moves with the hardware, the compiler, the power
state, and whatever else is running. This file is the dated record behind
the README's “tens of milliseconds” claim. The current table comes first,
then the protocol that produces a row, then the allocation contract CI
enforces, then the dated history every number came from.

## Current record

Every benchmark in the suite at its most recent measurement, on each machine
that has one, at the revision the column names. Point estimates only. The
dated entry a column names carries the 95% intervals and the prose. The M1
Pro column is the record the README and `docs/performance.md` cite. The
Xeon column is the first record on a second architecture. A dash is a row
that machine has not recorded.

| Measurement | Apple M1 Pro (2026-09-24, `af024e1`) | x86_64 Xeon, KVM (2026-09-26, `fa53c49`) |
| --- | ---: | ---: |
| `pixels/line_10k_80x20_iterm2` | — | 5.4976 ms |
| `pixels/line_10k_80x20_kitty` | — | 4.8146 ms |
| `pixels/line_10k_80x20_sixel` | — | 1.9946 ms |
| `plot/clone_12x5k_owned` | — | 12.463 µs |
| `plot/mapping_10m_80x20` | 2.0800 ms | 45.795 ms |
| `render/cells_2048x2048_80x24` | 40.555 ms | 42.234 ms |
| `render/color_by_100k/100000_categories` | 5.3475 ms | 8.0164 ms |
| `render/color_by_100k/100_categories` | — | 3.4750 ms |
| `render/color_by_100k/5_categories` | 2.0211 ms | 3.4446 ms |
| `render/encode_ansi_200x60` | — | 168.95 µs |
| `render/heatmap_64x48_80x24` | — | 433.51 µs |
| `render/layout_sweep_0_to_40` | — | 7.4483 ms |
| `render/line_10k_80x20` | 69.323 µs | 152.19 µs |
| `render/line_10m_80x20` | 30.714 ms | 142.37 ms |
| `render/scatter_1m_80x20` | — | 30.651 ms |
| `stat/bins_auto_1m` | — | 19.875 ms |
| `stat/fit_1m` | 5.1339 ms | 6.3555 ms |
| `stat/kde_1m_512` | — | 55.503 ms |
| `stat/m4_10m_160cols` | — | 126.67 ms |
| `stream/frame_512_100x20` | — | 50.868 µs |
| `ticks/linear(-1000000, 1000000, 10)` | — | 4.3600 µs |
| `ticks/linear(0, 100, 6)` | — | 1.2371 µs |
| `ticks/linear(0.001234, 0.005678, 8)` | — | 6.6754 µs |
| `ticks/linear(1.1, 8.7, 5)` | — | 2.5781 µs |
| `widget/dashboard_200x50` | 1.7717 ms | 4.1037 ms |
| `widget/hover_snap_10m_200x50` | 25.368 ms | 199.13 ms |
| `widget/zoom_10m_200x50` | 18.366 ms | 165.72 ms |

Both columns are 1.23.0. `fa53c49` is the release commit, and it differs
from `af024e1` only in version strings and the changelog.

## Recording protocol

A number enters this file the way a chart enters the docs: as program
output, with its provenance. To add or refresh a record:

1. Benchmark a machine that is otherwise idle. Never a shared CI runner; its
   wall-clock results are noise. Close what you can and note what you
   cannot. A virtual machine is disclosed as one.
2. Run the suites the row needs. The whole record is:

   ```sh
   cargo bench --bench render
   cargo bench --bench ticks
   cargo bench --bench widget --features ratatui
   cargo bench --bench pixels --features pixel
   ```

   A single row takes a Criterion filter, as the dated entries below show.
3. Print the block and paste it under a dated heading in the history:

   ```sh
   cargo run --example bench_record                    # every saved result
   cargo run --example bench_record -- render/line     # ids containing a filter
   ```

   It prints the revision (`-dirty` when the tree has uncommitted changes,
   so commit first), the machine, the OS, the compiler, the Criterion
   version and sample count, the measurement date, and one row per
   benchmark with Criterion's point estimate and 95% interval, in
   Criterion's own units and rounding. That is the `time:` line the bench
   printed. When Criterion compared the run against a saved previous one, a
   change column repeats its estimate.
4. Check repeatability before trusting a small row. Rerun the anchors
   (`render/line_10k_80x20`, `render/line_10m_80x20`) and quote Criterion's
   change line. A 95% interval describes one run's samples, not the next
   run.
5. Write the prose: what changed in the code, which rows moved and why, and
   which sat within noise. That is the one part a person writes.
6. Refresh the current-record table above and the summary in
   `docs/performance.md`. Keep the README's cited numbers true of the
   record they cite.

Allocation counts follow the contract below, not this protocol. They are
structural, CI gates them, and the ceilings change only when a reviewed
design change explains the new traffic.

## Allocation contract

The contract is structural: how many times a render touches the heap, and
how many bytes it asks for, counted by the harness's global allocator. It
does not depend on the clock and, as measured below, it does not depend on
the machine. Rust 1.88 is what CI trusts:

```sh
cargo bench --bench alloc              # print the counts
cargo bench --bench alloc -- --check   # enforce the ceilings CI runs
```

CI runs the check on Ubuntu 24.04 with Rust 1.88. It permits at most **275
allocations and 64 KiB of heap traffic** per measured render. The ceilings
leave room for compiler and allocator details, and they still catch a
structural regression: an allocation per input point, or a new large
intermediate buffer. CI does not gate wall-clock time on shared runners.

Measured on the x86_64 Xeon of the 2026-09-26 entry: both renders, at the
current release and at the revision the record held before, on the CI
compiler and on current stable.

| Render | Revision | Compiler | Allocations | Bytes | Output bytes |
| --- | --- | --- | ---: | ---: | ---: |
| `render/line_10k_80x20` | `fa53c49` (1.23.0) | 1.88.0 | 227 | 59,093 | 2,966 |
| | `fa53c49` | 1.94.1 | 223 | 58,861 | 2,966 |
| | `7bbc202` (1.17.0) | 1.88.0 | 187 | 58,620 | 2,966 |
| | `7bbc202` | 1.94.1 | 183 | 58,388 | 2,966 |
| `render/color_by_100k_unique_80x20` | `fa53c49` (1.23.0) | 1.88.0 | 92 | 34,342 | 1,791 |
| | `fa53c49` | 1.94.1 | 90 | 34,583 | 1,791 |
| | `7bbc202` (1.17.0) | 1.88.0 | 69 | 33,900 | 1,791 |
| | `7bbc202` | 1.94.1 | 67 | 34,141 | 1,791 |

The `7bbc202` rows on 1.94.1 reproduce the 2026-08-24 M1 Pro record
exactly: 183 allocations and 58,388 bytes for the line, 67 and 34,141 for
the categorical render. The count is a property of the code and the
compiler, not of the hardware or the OS. That is what makes it gateable.
Reading across the rows: a compiler version moves a count by four. The code
between 1.17.0 and 1.23.0 moved the line render by 40 allocations and 473
bytes, and the categorical render by 23 allocations, for byte-identical
output.

A bisect on this machine puts the whole line-render jump on one commit,
`6a8b287` (Ticks take units and whole-number steps, in 1.23.0): 186
allocations before it, 223 after, on 1.94.1. Every tick label is now built
by formatting its already-formatted numeric text together with a unit
prefix into a fresh `String`, once per label per layout pass, where the
numeric text used to be the label. That is a per-label cost, a fixed
handful per axis, not a per-point one. It is a candidate for the next
cleanup, not a ceiling change. Neither growth is per point; the bytes
barely moved. But the headroom under the CI ceiling on the CI compiler is
now 48 allocations, down from 88, and the next feature that adds a handful
per axis or per legend entry will spend it. Change the ceilings only when a
reviewed design change explains the new traffic.

The line measurement includes gap-aware M4 state and the two-color cell
surface. The categorical measurement shows that rendering does not allocate
per point or per category. Labels and identities are kept when the mark is
built, and render preparation borrows them. The 64 KiB ceiling is unchanged,
and it still catches a larger per-cell representation or a manufactured
intermediate.

## History

Newest first. Each entry names the revision, machine, compiler, and commands
behind its rows.

### 2026-09-26 addition, a second architecture (1.23.0)

- Revision: `fa53c49` (the 1.23.0 release commit)
- Machine: Intel Xeon @ 2.10 GHz, 4 cores, 15 GiB RAM. A KVM guest, otherwise
  idle (load average 0.05 before the run): the cloud container this record
  was written in
- OS: Ubuntu 24.04.4 LTS (Linux 6.18.44), x86_64
- Compiler: `rustc 1.94.1 (e408947bf 2026-03-25)`, LLVM 21.1.8
- Profile: Cargo `bench` / optimized, Criterion 0.5.1, 100 samples per row

| Measurement | Estimate | 95% interval |
| --- | ---: | ---: |
| `pixels/line_10k_80x20_iterm2` | 5.4976 ms | 5.3732–5.6377 ms |
| `pixels/line_10k_80x20_kitty` | 4.8146 ms | 4.7073–4.9277 ms |
| `pixels/line_10k_80x20_sixel` | 1.9946 ms | 1.9602–2.0300 ms |
| `plot/clone_12x5k_owned` | 12.463 µs | 12.343–12.610 µs |
| `plot/mapping_10m_80x20` | 45.795 ms | 45.456–46.126 ms |
| `render/cells_2048x2048_80x24` | 42.234 ms | 41.741–42.761 ms |
| `render/color_by_100k/100000_categories` | 8.0164 ms | 7.8695–8.1892 ms |
| `render/color_by_100k/100_categories` | 3.4750 ms | 3.4116–3.5497 ms |
| `render/color_by_100k/5_categories` | 3.4446 ms | 3.3729–3.5256 ms |
| `render/encode_ansi_200x60` | 168.95 µs | 166.36–171.60 µs |
| `render/heatmap_64x48_80x24` | 433.51 µs | 423.14–444.78 µs |
| `render/layout_sweep_0_to_40` | 7.4483 ms | 7.2899–7.6139 ms |
| `render/line_10k_80x20` | 152.19 µs | 147.25–156.95 µs |
| `render/line_10m_80x20` | 142.37 ms | 140.82–143.92 ms |
| `render/scatter_1m_80x20` | 30.651 ms | 30.274–31.119 ms |
| `stat/bins_auto_1m` | 19.875 ms | 19.627–20.149 ms |
| `stat/fit_1m` | 6.3555 ms | 6.2636–6.4479 ms |
| `stat/kde_1m_512` | 55.503 ms | 54.684–56.368 ms |
| `stat/m4_10m_160cols` | 126.67 ms | 125.56–127.86 ms |
| `stream/frame_512_100x20` | 50.868 µs | 49.468–52.305 µs |
| `ticks/linear(-1000000, 1000000, 10)` | 4.3600 µs | 4.3181–4.4089 µs |
| `ticks/linear(0, 100, 6)` | 1.2371 µs | 1.1927–1.2876 µs |
| `ticks/linear(0.001234, 0.005678, 8)` | 6.6754 µs | 6.4222–6.9449 µs |
| `ticks/linear(1.1, 8.7, 5)` | 2.5781 µs | 2.4966–2.6650 µs |
| `widget/dashboard_200x50` | 4.1037 ms | 3.9878–4.2311 ms |
| `widget/hover_snap_10m_200x50` | 199.13 ms | 198.07–200.23 ms |
| `widget/zoom_10m_200x50` | 165.72 ms | 164.04–167.94 ms |

```sh
cargo bench --bench render
cargo bench --bench ticks
cargo bench --bench widget --features ratatui
cargo bench --bench pixels --features pixel
cargo run --example bench_record
```

The first record on a machine that is not the M1 Pro, and the first to run
the whole suite. Seventeen of these rows had a bench and no record until
now. The code is the same 1.23.0 the entry below measured, so the two
columns of the current table compare architectures, not revisions.

What the comparison says. The per-bucket and per-cell work costs about the
same on both machines. The cells row is within 4% of the M1 Pro, the fit
within 24%, the categorical pair within 1.7×. The per-point walks are where
the machines part. The 10k line is 2.2× slower here, the ten-million-point
line 4.6×. The M4 reduction alone (`stat/m4_10m_160cols`, 127 of the
142 ms) costs about 12.7 ns per point, against roughly 3 ns on the M1 Pro.
The extent probe behind `plot/mapping_10m_80x20` is 22× slower. The hover
cursor's nearest scan, the 33 ms between the hover and zoom rows, costs
about 3.3 ns per point against 0.6 ns. Here the zoomed frame does not
undercut the full-view render, as it does on the M1 Pro. The column test
that rejects out-of-window points early saves less than the per-point cost
it sits behind. The record states these ratios and does not explain them.
Codegen for the baseline `x86-64` target against the M1's NEON, and the
guest's memory system, are the two places to look.

Repeatability, measured. Rerunning the two anchors right after the suite
put `render/line_10m_80x20` at 143.00 ms (Criterion: +0.44%, "no change")
and `render/line_10k_80x20` at 178.39 µs (+12.46%, p < 0.05). On this guest
the millisecond rows repeat within a percent. The microsecond rows' 95%
intervals understate their run-to-run spread by an order of magnitude. Read
the small rows here as ±15%, and do not read a sub-millisecond change on
this machine as a regression without a third run.

Allocation counts, the structural contract, were measured on this machine
on both compilers and recorded in the contract section above.

### 2026-09-24 addition (released in 1.23.0)

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

### 2026-08-28 addition — the mapping pass (released in 1.20.0)

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

### 2026-08-28 addition (released in 1.20.0)

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

### 2026-08-25 addition (released in 1.18.0)

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

### 2026-08-24 baseline (1.17.0)

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

### 2026-08-15 baseline (1.16.0)

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

### 2026-08-07 baseline

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

#### Profiling decision

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
