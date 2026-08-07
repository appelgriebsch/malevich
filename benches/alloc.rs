//! Counting-allocator harness: prints heap traffic for one end-to-end render.
//!
//! Not a timing bench — a measurement of the allocation baseline, so the
//! aggregation work can show its zero-alloc-per-point contract with numbers. Pass
//! `--check` to enforce the deliberately generous CI budgets below.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNT: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);

// Structural regression fences, not optimization targets. The recorded Rust 1.88
// baseline is comfortably below both; headroom absorbs allocator/platform details
// while still catching an accidental allocation per point or a large new buffer.
const MAX_ALLOCATIONS: usize = 450;
const MAX_ALLOCATED_BYTES: usize = 64 * 1024;

struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        COUNT.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

fn main() {
    let values: Vec<f64> = (0..10_000).map(|i| (i as f64 * 0.01).sin()).collect();
    let frame = malevich::Frame::plain(80, 20);
    let plot = malevich::line(&values[..]);
    let _warmup = plot.render(&frame);

    let count_before = COUNT.load(Ordering::Relaxed);
    let bytes_before = BYTES.load(Ordering::Relaxed);
    let output = plot.render(&frame);
    let count = COUNT.load(Ordering::Relaxed) - count_before;
    let bytes = BYTES.load(Ordering::Relaxed) - bytes_before;

    println!(
        "render/line_10k_80x20: {count} allocations, {bytes} bytes heap traffic, {} output bytes",
        output.len()
    );

    if std::env::args().any(|argument| argument == "--check") {
        assert!(
            count <= MAX_ALLOCATIONS,
            "allocation count exceeded its budget: {count} > {MAX_ALLOCATIONS}"
        );
        assert!(
            bytes <= MAX_ALLOCATED_BYTES,
            "allocated bytes exceeded their budget: {bytes} > {MAX_ALLOCATED_BYTES}"
        );
    }
}
