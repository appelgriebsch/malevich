# Live charts

Live data stays at the edge of the crate, in `malevich::stream`, and the module is small on purpose. A sliding window shared across threads. A counter-to-rate helper. An in-place repaint that never owns the screen. The core stays pure — every render that runs is still a pure function of its inputs — and time enters only at this rim.

## The window

`Ring::new(capacity)` is a sliding window. Push readings from a sampler thread, snapshot from the UI thread, and render the snapshot. It is the one lock in the library. `Ring::growing` keeps every value since the start, with no fixed window. `Rate` turns a monotonic counter — bytes received, requests served — into per-interval deltas.

```rust
use std::sync::Arc;
use std::time::Duration;
use malevich::stream::{Live, Ring};
use malevich::{Frame, Line, Plot};

let ring = Arc::new(Ring::new(240));
let producer = Arc::clone(&ring);
std::thread::spawn(move || loop {
    producer.push(read_sensor());
    std::thread::sleep(Duration::from_millis(100));
});

let mut live = Live::detect(std::io::stdout());
loop {
    let values = ring.snapshot();
    let plot = Plot::new().layer(Line::y(&values[..])).title("sensor");
    live.draw(&plot, &Frame::detect())?;
    std::thread::sleep(Duration::from_millis(100));
}
```

## The repaint

`Live::draw(&plot, &frame)` renders and repaints in place. Cursor up, erase down, one buffered write, bracketed as a synchronized-output frame so the terminal swaps the whole chart at once. No alternate screen, so the final frame stays in your scrollback. Ctrl-C restores the cursor. `Live::detect` repaints only on a terminal. When the destination is a pipe or a file, frames append as plain text, and no escape byte is written where it is not safe.

`cargo run --example live` is the moving version of the showcase tour.

## Follow the stream

For a series that keeps growing, the `Viewport` has `tail(latest, width)`: a window ending at the latest x, `width` wide. The reduction re-aggregates to that window on every frame, so a ten-million-point history tails at full speed.

{{figure stream_tail}}

## From the shell

`kaz --live` reads stdin forever, one value per line, and repaints a sliding line. `--window N` sets the window, `--fps N` the repaint rate, and `--rate` plots the per-interval delta of a counter. Where the destination is not a terminal, the frames append as text.

```sh
ping -i.2 host | grep -oE 'time=[0-9.]+' | tr -d 'time=' | kaz line --live -t ping
vmstat 1 | awk 'NR>2{print $1}' | kaz line --live -t runnable
```

If a live plot looks frozen, the *producer* is buffering — pipes hold output until a block fills. Unbuffer at the source: `stdbuf -oL producer`, `grep --line-buffered`, or `awk '{print; fflush()}'`.

## The demos

[`sysmon`](https://github.com/shergin/malevich/tree/main/demos/sysmon) is the streaming story as an app. A sampler thread pushes CPU, memory, and network readings into `Ring` windows, network counters through `Rate`. The ratatui UI snapshots and redraws four times a second. Filled areas are pinned to honest domains: CPU to 0–100, memory to the machine's total, network on an SI-prefixed bytes-per-second axis. A per-core heatmap carries a colorbar.

```sh
cargo run -p sysmon
```

## What it will not do

No animations, no event loop, no raw mode. The library writes one frame when you ask it to, and it never reads input. A host that wants a schedule owns the loop. A host that wants gestures owns the mouse ([interaction](../interaction/)). That boundary is what lets the same plot value render in a ratatui dashboard, an Ink app, a notebook, and a CI log, with no mode to switch.
