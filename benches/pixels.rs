//! Regression fence for pixel rendering: the hybrid path plus each protocol
//! encoder, a 10k-point line into an 80×20 frame at a 10×20 cell — a panel of
//! roughly 800×340 device pixels, resolved, rasterized, and encoded per pass.

use criterion::{Criterion, criterion_group, criterion_main};
use malevich::Frame;
use malevich::pixel::{Graphics, Protocol};
use std::hint::black_box;

fn pixel_render(c: &mut Criterion) {
    let values: Vec<f64> = (0..10_000)
        .map(|i| (i as f64 * 0.01).sin() * (i as f64).sqrt())
        .collect();
    let frame = Frame::plain(80, 20);
    for (name, protocol) in [
        ("sixel", Protocol::Sixel),
        ("kitty", Protocol::Kitty),
        ("iterm2", Protocol::ITerm2),
    ] {
        let graphics = Graphics::new(protocol).cell_size(10, 20);
        c.bench_function(&format!("pixels/line_10k_80x20_{name}"), |b| {
            b.iter(|| {
                black_box(malevich::line(black_box(&values[..])).render_pixels(&frame, &graphics))
            });
        });
    }
}

criterion_group!(benches, pixel_render);
criterion_main!(benches);
