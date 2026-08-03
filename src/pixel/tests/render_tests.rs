use crate::pixel::{Graphics, Protocol};
use crate::plot::Frame;
use crate::{Line, Plot, Points};

fn graphics() -> Graphics {
    Graphics::new(Protocol::Sixel).cell_size(4, 8)
}

fn sample() -> Plot<'static> {
    let x: Vec<f64> = (0..32).map(f64::from).collect();
    let y: Vec<f64> = x.iter().map(|v| (v * 0.4).sin()).collect();
    Plot::new()
        .layer(Line::xy(x.clone(), y.clone()).label("wave"))
        .layer(Points::xy(x, y))
        .title("hybrid")
}

#[test]
fn hybrid_output_weaves_text_chrome_around_a_sixel_panel() {
    let out = sample().render_pixels(&Frame::plain(40, 12), &graphics());
    // The chrome is ordinary text…
    assert!(out.contains("hybrid"), "title missing");
    // …the panel is a sixel payload bracketed by cursor save/restore…
    assert!(out.contains("\x1b7"), "missing DECSC");
    assert!(out.contains("\x1bP0;1;0q"), "missing sixel introducer");
    assert!(out.ends_with("\x1b8"), "missing DECRC at the end");
    // …reached by relative movement, never absolute addressing.
    assert!(
        out.contains("[9A") || out.contains("[10A"),
        "missing cursor-up: {:?}",
        &out[out.len().min(200)..]
    );
    assert!(
        !out.contains("\x1b[H"),
        "absolute addressing is scroll-unsafe"
    );
}

#[test]
fn marks_ink_the_panel_image_not_the_text_grid() {
    let out = sample().render_pixels(&Frame::plain(40, 12), &graphics());
    let text = &out[..out.find('\x1b').expect("a sixel payload follows the text")];
    // Braille (or any subpixel ink) would mean marks leaked onto the cell grid.
    assert!(
        !text.chars().any(|c| ('\u{2800}'..='\u{28FF}').contains(&c)),
        "marks drew on the text grid"
    );
}

#[test]
fn pixel_rendering_is_deterministic() {
    let (a, b) = (
        sample().render_pixels(&Frame::plain(40, 12), &graphics()),
        sample().render_pixels(&Frame::plain(40, 12), &graphics()),
    );
    assert_eq!(a, b);
}

#[test]
fn a_zero_cell_size_degrades_to_text_only() {
    let gfx = Graphics::new(Protocol::Sixel).cell_size(0, 8);
    let out = sample().render_pixels(&Frame::plain(40, 12), &gfx);
    assert!(out.contains("hybrid"));
    assert!(
        !out.contains("\x1bP"),
        "no image payload without a cell size"
    );
}

#[test]
fn an_empty_frame_renders_to_nothing_and_does_not_panic() {
    let out = sample().render_pixels(&Frame::plain(0, 0), &graphics());
    assert!(!out.contains("\x1bP"));
}

#[test]
fn the_corners_style_falls_back_to_a_pixel_line() {
    let x: Vec<f64> = (0..16).map(f64::from).collect();
    let y: Vec<f64> = x.iter().map(|v| v * 0.5).collect();
    let plot = Plot::new().layer(Line::xy(x, y).style(crate::mark::LineStyle::Corners));
    let out = plot.render_pixels(&Frame::plain(40, 12), &graphics());
    let text = &out[..out.find('\x1b').expect("a sixel payload follows the text")];
    for corner in ['\u{256D}', '\u{256E}', '\u{256F}', '\u{2570}'] {
        assert!(
            !text.contains(corner),
            "corner glyph {corner} leaked into hybrid output"
        );
    }
    assert!(out.contains("\x1bP0;1;0q"));
}
