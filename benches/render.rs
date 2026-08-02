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

fn scatter_render(c: &mut Criterion) {
    let n = 1_000_000;
    let x: Vec<f64> = (0..n).map(|i| (i as f64 * 0.417).sin()).collect();
    let y: Vec<f64> = (0..n).map(|i| (i as f64 * 0.731).cos()).collect();
    let frame = Frame::plain(80, 20);
    c.bench_function("render/scatter_1m_80x20", |b| {
        b.iter(|| {
            black_box(malevich::scatter(black_box(&x[..]), black_box(&y[..])).render(&frame))
        });
    });
}

fn ansi_encoding(c: &mut Criterion) {
    use malevich::{Charset, Color, ColorMode, render::Surface};
    let mut surface = Surface::new(200, 60, Charset::Braille);
    for i in 0..(200 * 2) {
        let color = if i % 3 == 0 { Color::Red } else { Color::Cyan };
        surface.line((i as f64, 0.0), ((400 - i) as f64, 239.0), color);
    }
    c.bench_function("render/encode_ansi_200x60", |b| {
        b.iter(|| black_box(surface.encode(black_box(ColorMode::Ansi16))));
    });
}

fn ten_million_points(c: &mut Criterion) {
    let n = 10_000_000;
    let y: Vec<f64> = (0..n)
        .map(|i| (i as f64 * 0.0002).sin() * (i as f64 * 0.000013).cos() * 8.0)
        .collect();
    let frame = Frame::plain(80, 20);
    // The headline fence: end to end, ingestion through ANSI-free encoding, with
    // the automatically inserted M4 doing the heavy lifting.
    c.bench_function("render/line_10m_80x20", |b| {
        b.iter(|| black_box(malevich::line(black_box(&y[..])).render(&frame)));
    });
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    c.bench_function("stat/m4_10m_160cols", |b| {
        b.iter(|| {
            black_box(malevich::stat::m4(
                black_box(&x[..]),
                black_box(&y[..]),
                160,
            ))
        });
    });
}

fn histogram_binning(c: &mut Criterion) {
    let values: Vec<f64> = (0..1_000_000)
        .map(|i| {
            let i = i as f64;
            ((i * 0.731).sin() + (i * 1.13).sin()) * 4.0 + 20.0
        })
        .collect();
    c.bench_function("stat/bins_auto_1m", |b| {
        b.iter(|| black_box(malevich::stat::Bins::auto(black_box(&values[..]), 60)));
    });
}

criterion_group!(
    benches,
    line_render,
    scatter_render,
    ansi_encoding,
    ten_million_points,
    histogram_binning
);
criterion_main!(benches);
