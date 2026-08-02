//! A ratatui dashboard embedding live malevich widgets. Run with
//! `cargo run --example tui --features ratatui`; press `q` to quit.

use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode};
use malevich::stream::Ring;
use malevich::{Line, Plot};
use ratatui::layout::{Constraint, Layout};

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let ring = Ring::new(200);
    let producer = ring.clone();
    std::thread::spawn(move || {
        let start = Instant::now();
        loop {
            let t = start.elapsed().as_secs_f64();
            producer.push(20.0 + (t * 2.0).sin() * 6.0 + (t * 7.7).sin() * 2.0);
            std::thread::sleep(Duration::from_millis(40));
        }
    });

    let histogram_data: Vec<f64> = (0..3000)
        .map(|i| {
            let i = f64::from(i);
            ((i * 0.731).sin() + (i * 1.13).sin() + (i * 2.71).sin()) * 2.0 + 10.0
        })
        .collect();

    loop {
        let snapshot = ring.snapshot();
        terminal.draw(|frame| {
            let [top, bottom] = Layout::vertical([Constraint::Percentage(55), Constraint::Fill(1)])
                .areas(frame.area());
            let live = Plot::new()
                .layer(Line::y(&snapshot[..]).label("metric"))
                .title("live (q quits)");
            frame.render_widget(live.widget(), top);
            let hist = malevich::hist(&histogram_data[..]).title("distribution");
            frame.render_widget(hist.widget(), bottom);
        })?;
        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
            && key.code == KeyCode::Char('q')
        {
            break;
        }
    }
    ratatui::restore();
    Ok(())
}
