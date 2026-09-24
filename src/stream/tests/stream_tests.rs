use super::{Live, Rate, Ring};
use crate::{Frame, Plot};

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<Ring>();

#[test]
fn the_ring_slides_and_clones_share_the_window() {
    let ring = Ring::new(3);
    let producer = ring.clone();
    for value in [1.0, 2.0, 3.0, 4.0] {
        producer.push(value);
    }
    assert_eq!(ring.snapshot(), [2.0, 3.0, 4.0]);
    assert_eq!(ring.len(), 3);
}

#[test]
fn pushes_from_another_thread_arrive() {
    let ring = Ring::new(8);
    let producer = ring.clone();
    std::thread::spawn(move || {
        for i in 0..5 {
            producer.push(f64::from(i));
        }
    })
    .join()
    .unwrap();
    assert_eq!(ring.len(), 5);
}

#[test]
fn rates_are_deltas_with_honest_gaps() {
    let mut rate = Rate::new();
    assert!(rate.delta(10.0).is_nan());
    assert_eq!(rate.delta(15.0), 5.0);
    assert!(rate.delta(f64::NAN).is_nan());
    assert!(rate.delta(20.0).is_nan());
    assert_eq!(rate.delta(22.0), 2.0);
}

#[test]
fn the_first_draw_prints_and_later_draws_repaint_in_place() {
    let plot = Plot::new().layer(crate::Line::y(&[1.0, 2.0][..]));
    let frame = Frame::plain(20, 5);
    let mut out = Vec::new();
    {
        let mut live = Live::new(&mut out);
        assert!(live.repaints());
        live.draw(&plot, &frame).unwrap();
        live.draw(&plot, &frame).unwrap();
    }
    let text = String::from_utf8(out).unwrap();
    let mut frames = text.split("\x1b[5A\r\x1b[J");
    let first = frames.next().unwrap();
    let unbracketed = first.replace("\x1b[?2026h", "").replace("\x1b[?2026l", "");
    assert!(
        !unbracketed.contains('\x1b'),
        "first draw must not move the cursor: {first:?}"
    );
    assert!(frames.next().is_some(), "second draw must move up 5 rows");
    // Every frame is one synchronized-output bracket.
    assert_eq!(text.matches("\x1b[?2026h").count(), 2);
    assert_eq!(text.matches("\x1b[?2026l").count(), 2);
    assert!(text.starts_with("\x1b[?2026h"));
    assert!(text.ends_with("\x1b[?2026l"));
}

#[test]
fn a_detected_file_receives_plain_frames_and_no_escape_bytes() {
    let plot = Plot::new().layer(crate::Line::y(&[1.0, 2.0][..]));
    let frame = Frame::plain(20, 5);
    let path = std::env::temp_dir().join(format!(
        "malevich-live-{}-{}.log",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    ));
    {
        let file = std::fs::File::create(&path).unwrap();
        let mut live = Live::detect(file);
        assert!(!live.repaints(), "a file is not a terminal");
        live.draw(&plot, &frame).unwrap();
        live.draw(&plot, &frame).unwrap();
    }
    let text = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    assert!(!text.contains('\x1b'), "a file got escape bytes: {text:?}");
    assert_eq!(text.matches(plot.render(&frame).as_str()).count(), 2);
}
