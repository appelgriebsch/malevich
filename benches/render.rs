//! Regression fence for end-to-end rendering: preset construction, resolve, layout,
//! rasterization, and encoding of a 10k-point line into an 80×20 frame.

use criterion::{Criterion, criterion_group, criterion_main};
use malevich::Frame;
use std::hint::black_box;

fn line_render(c: &mut Criterion) {
    let values: Vec<f64> = (0..10_000)
        .map(|i| (i as f64 * 0.01).sin() * (i as f64).sqrt())
        .collect();
    let frame = Frame::plain(80, 20);
    c.bench_function("render/line_10k_80x20", |b| {
        b.iter(|| black_box(malevich::line(black_box(&values[..])).render(&frame)));
    });
}

criterion_group!(benches, line_render);
criterion_main!(benches);
