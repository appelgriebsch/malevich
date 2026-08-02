//! A live chart: a producer thread pushes into a shared ring while this thread
//! repaints in place — cursor up, erase, redraw, one write per frame. Run it in a
//! terminal; it draws two hundred frames and stops.

use std::time::Duration;

use malevich::stream::{Live, Ring};
use malevich::{Frame, Line, Plot};

fn main() -> std::io::Result<()> {
    let ring = Ring::new(240);
    let producer = ring.clone();
    let worker = std::thread::spawn(move || {
        for i in 0..2_000u32 {
            let t = f64::from(i) * 0.05;
            producer.push(20.0 + (t * 0.7).sin() * 6.0 + (t * 2.3).sin() * 2.0);
            std::thread::sleep(Duration::from_millis(5));
        }
    });

    let mut live = Live::new(std::io::stderr());
    for _ in 0..200 {
        let snapshot = ring.snapshot();
        let chart = Plot::new()
            .layer(Line::y(&snapshot[..]))
            .title("a live metric (synthetic)");
        live.draw(&chart, &Frame::detect())?;
        std::thread::sleep(Duration::from_millis(50));
    }
    worker.join().expect("producer thread");
    Ok(())
}
