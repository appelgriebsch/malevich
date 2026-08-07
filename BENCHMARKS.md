# Benchmark baselines

Malevich treats performance as a measured engineering constraint, not a portable
speed promise. Wall-clock results vary with hardware, compiler, power state, and
background load. This file is the authoritative dated record behind the README's
“tens of milliseconds” claim.

## 2026-08-07 baseline

- Revision: `0f3ad5a`
- Machine: 2021 MacBook Pro, Apple M1 Pro (10 cores), 32 GB RAM
- OS: macOS 26.5.2 (Darwin 25.5.0), arm64
- Compiler: `rustc 1.97.1 (8bab26f4f 2026-07-14)`, LLVM 22.1.8
- Profile: Cargo `bench` / optimized, Criterion 0.5 default 100-sample run

| Measurement | Estimate | 95% interval |
| --- | ---: | ---: |
| `render/line_10k_80x20` | 81.818 µs | 81.756–81.878 µs |
| `render/line_10m_80x20` | 42.260 ms | 42.224–42.300 ms |

Commands:

```sh
cargo bench --bench render -- render/line_10k_80x20
cargo bench --bench render -- render/line_10m_80x20
```

The benchmark is end to end: construct the preset, resolve domains and layout,
perform M4 reduction, rasterize an 80×20 braille frame, and encode the final string.
It is single-threaded. The ten-million-point input vectors are prepared outside the
timed iteration.

## Allocation contract

The same revision, optimized on the machine above, measured the 10k render at **356
allocations and 51,954 allocated bytes**, producing 2,966 output bytes:

```sh
cargo bench --bench alloc
```

CI runs that harness on Ubuntu 24.04 with Rust 1.88 and `--check`. It permits at most
450 allocations and 64 KiB of heap traffic. Those ceilings intentionally leave
headroom for compiler and allocator details while catching structural regressions
such as an allocation per input point or a new large intermediate buffer. CI does not
gate wall-clock time on shared runners.

To update this record, benchmark an otherwise idle machine, record the revision,
hardware, OS, compiler, commands, point estimates, and confidence intervals, then
change the allocation ceilings only when a reviewed design change explains the new
traffic.
