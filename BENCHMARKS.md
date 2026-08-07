# Benchmark baselines

Malevich treats performance as a measured engineering constraint, not a portable
speed promise. Wall-clock results vary with hardware, compiler, power state, and
background load. This file is the authoritative dated record behind the README's
“tens of milliseconds” claim.

## 2026-08-07 baseline

- Revision: `4962935`
- Machine: 2021 MacBook Pro, Apple M1 Pro (10 cores), 32 GB RAM
- OS: macOS 26.5.2 (Darwin 25.5.0), arm64
- Compiler: `rustc 1.97.1 (8bab26f4f 2026-07-14)`, LLVM 22.1.8
- Profile: Cargo `bench` / optimized, Criterion 0.5 default 100-sample run

| Measurement | Estimate | 95% interval |
| --- | ---: | ---: |
| `render/line_10k_80x20` | 68.197 µs | 68.118–68.283 µs |
| `render/line_10m_80x20` | 36.469 ms | 36.426–36.514 ms |

Commands:

```sh
cargo bench --bench render -- render/line_10k_80x20
cargo bench --bench render -- render/line_10m_80x20
```

The benchmark is end to end: construct the preset, resolve domains and layout,
perform M4 reduction, rasterize an 80×20 braille frame, and encode the final string.
It is single-threaded. The ten-million-point input vectors are prepared outside the
timed iteration.

The earlier `0f3ad5a` record on this machine was 81.818 µs and 42.260 ms,
respectively. The current measurements are 16.6% and 13.7% lower. These are
same-machine historical comparisons, not portable performance promises.

### Profiling decision

A five-second Instruments Time Profiler capture of the 10k case attributed 2,670
of 5,108 leaf samples (about 52%) to resolution. A measured A/B implementation
kept the compact resolved-layer probe rather than duplicating every mark's domain
rules in a parallel metadata type: the smaller design was faster and retains one
source of truth. The accepted change instead:

- keeps implicit coordinates symbolic;
- summarizes a line into only the two endpoints needed by its linear or log axis;
- retains the probed layout for drawing, avoiding a second round of tick formatting,
  gutter measurement, and colorbar work.

The pixel-exact raw-versus-M4 oracle and all rendering snapshots remained identical.

## Allocation contract

The same revision, optimized on the machine above, measured the 10k render at **183
allocations and 49,508 allocated bytes**, producing 2,966 output bytes. Rust 1.88
reported the same figures:

```sh
cargo bench --bench alloc
```

CI runs that harness on Ubuntu 24.04 with Rust 1.88 and `--check`. It permits at most
275 allocations and 64 KiB of heap traffic. Those ceilings intentionally leave
headroom for compiler and allocator details while catching structural regressions
such as an allocation per input point or a new large intermediate buffer. CI does not
gate wall-clock time on shared runners.

To update this record, benchmark an otherwise idle machine, record the revision,
hardware, OS, compiler, commands, point estimates, and confidence intervals, then
change the allocation ceilings only when a reviewed design change explains the new
traffic.
