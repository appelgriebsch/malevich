//! Regression fence for tick placement speed. The claim: placing ticks is always
//! microseconds — never a visible cost next to rendering.

use criterion::{Criterion, criterion_group, criterion_main};
use malevich::scale::Ticks;
use std::hint::black_box;

fn tick_placement(c: &mut Criterion) {
    let cases: [(f64, f64, usize); 4] = [
        (0.0, 100.0, 6),
        (0.001_234, 0.005_678, 8),
        (-1e6, 1e6, 10),
        (1.1, 8.7, 5),
    ];
    let mut group = c.benchmark_group("ticks");
    for (lo, hi, target) in cases {
        group.bench_function(format!("linear({lo}, {hi}, {target})"), |b| {
            b.iter(|| Ticks::linear(black_box(lo), black_box(hi), black_box(target)));
        });
    }
    group.finish();
}

criterion_group!(benches, tick_placement);
criterion_main!(benches);
