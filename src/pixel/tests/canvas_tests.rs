use crate::render::{Canvas, Color, PlotRect};

use super::PixelCanvas;

const RED: Color = Color::Rgb(255, 0, 0);

fn rect() -> PlotRect {
    PlotRect {
        gutter: 0,
        top: 0,
        columns: 4,
        rows: 4,
    }
}

#[test]
fn a_new_canvas_is_fully_transparent() {
    let canvas = PixelCanvas::new(4, 4, (8, 16));
    assert_eq!(canvas.size(), (32, 64));
    for y in 0..64 {
        for x in 0..32 {
            assert_eq!(canvas.get(x, y), None);
        }
    }
}

#[test]
fn dot_sets_the_nearest_pixel_and_clips_outside() {
    let mut canvas = PixelCanvas::new(2, 2, (8, 8));
    canvas.dot(3.4, 5.6, RED);
    assert_eq!(canvas.get(3, 6), Some(RED));
    canvas.dot(-1.0, 0.0, RED);
    canvas.dot(1000.0, 0.0, RED);
    canvas.dot(f64::NAN, 0.0, RED);
    let drawn = (0..16)
        .flat_map(|y| (0..16).map(move |x| (x, y)))
        .filter(|&(x, y)| canvas.get(x, y).is_some())
        .count();
    assert_eq!(drawn, 1);
}

#[test]
fn a_horizontal_line_fills_every_pixel_between_its_endpoints() {
    let mut canvas = PixelCanvas::new(4, 1, (8, 8));
    canvas.line((2.0, 3.0), (20.0, 3.0), RED);
    for x in 2..=20 {
        assert_eq!(canvas.get(x, 3), Some(RED), "x={x}");
    }
    assert_eq!(canvas.get(1, 3), None);
    assert_eq!(canvas.get(21, 3), None);
}

#[test]
fn lines_respect_the_clip_rectangle() {
    let mut canvas = PixelCanvas::new(4, 1, (8, 8));
    canvas.set_clip(8, 0, 16, 8);
    canvas.line((0.0, 4.0), (31.0, 4.0), RED);
    for x in 0..32 {
        let expected = (8..16).contains(&x);
        assert_eq!(canvas.get(x as usize, 4).is_some(), expected, "x={x}");
    }
    canvas.clear_clip();
    canvas.line((0.0, 5.0), (31.0, 5.0), RED);
    assert_eq!(canvas.get(0, 5), Some(RED));
    assert_eq!(canvas.get(31, 5), Some(RED));
}

#[test]
fn a_bar_fills_its_exact_rectangle_from_the_baseline() {
    let mut canvas = PixelCanvas::new(4, 4, (8, 8));
    // Plot-local: a bar over x ∈ [4, 12), from the value end at y=8 down to the
    // baseline at y=24.
    canvas.bar((4.0, 12.0), 8.0, 24.0, true, rect(), RED);
    for y in 8..24 {
        for x in 4..12 {
            assert_eq!(canvas.get(x, y), Some(RED), "({x}, {y})");
        }
    }
    assert_eq!(canvas.get(3, 8), None);
    assert_eq!(canvas.get(12, 8), None);
    assert_eq!(canvas.get(4, 7), None);
    assert_eq!(canvas.get(4, 24), None);
}

#[test]
fn a_zero_width_bar_still_draws_one_pixel_column() {
    let mut canvas = PixelCanvas::new(4, 4, (8, 8));
    canvas.bar((6.2, 6.4), 0.0, 4.0, true, rect(), RED);
    assert_eq!(canvas.get(6, 2), Some(RED));
}

#[test]
fn the_gutter_offsets_bars_into_the_plot_rectangle() {
    let mut canvas = PixelCanvas::new(4, 4, (8, 8));
    let offset = PlotRect {
        gutter: 2,
        top: 1,
        columns: 2,
        rows: 3,
    };
    canvas.bar((0.0, 4.0), 0.0, 8.0, true, offset, RED);
    // gutter 2 cells × 8 px, top 1 cell × 8 px.
    assert_eq!(canvas.get(16, 8), Some(RED));
    assert_eq!(canvas.get(15, 8), None);
    assert_eq!(canvas.get(16, 7), None);
}

#[test]
fn the_marker_clears_a_band_through_a_fill() {
    let mut canvas = PixelCanvas::new(4, 4, (8, 8));
    canvas.bar((0.0, 32.0), 0.0, 32.0, true, rect(), RED);
    canvas.marker(16.0, 8.0, 16.0, RED);
    assert_eq!(canvas.get(16, 16), None);
    assert_eq!(canvas.get(8, 16), None);
    assert_eq!(canvas.get(24, 16), None);
    // Above and below the band the fill survives.
    assert_eq!(canvas.get(16, 12), Some(RED));
    assert_eq!(canvas.get(16, 20), Some(RED));
    // Outside the reach the fill survives.
    assert_eq!(canvas.get(4, 16), Some(RED));
}

#[test]
fn patches_are_single_pixels_inside_the_plot_rectangle() {
    let mut canvas = PixelCanvas::new(4, 4, (8, 8));
    assert_eq!(canvas.patch_size(), (1, 1));
    let offset = PlotRect {
        gutter: 1,
        top: 1,
        columns: 3,
        rows: 3,
    };
    canvas.patch(3, 5, offset, 0.5, RED);
    assert_eq!(canvas.get(11, 13), Some(RED));
    assert_eq!(canvas.get(10, 13), None);
}

#[test]
fn text_blits_the_baked_font_and_skips_what_it_lacks() {
    let mut canvas = PixelCanvas::new(4, 1, (8, 8));
    canvas.text(0, 0, "A", RED);
    let ink = |canvas: &PixelCanvas, x0: usize| {
        (0..8)
            .flat_map(|y| (0..8).map(move |x| (x, y)))
            .filter(|&(x, y)| canvas.get(x0 + x, y).is_some())
            .count()
    };
    let drawn = ink(&canvas, 0);
    assert!(drawn > 10, "letter A should ink a good part of its cell");
    // The glyph never leaks outside its cell at scale 1.
    assert_eq!(ink(&canvas, 8), 0);
    // Unsupported glyphs advance without ink: the é draws nothing, the B that
    // follows it lands one cell further right.
    canvas.text(1, 0, "\u{e9}B", RED);
    assert_eq!(ink(&canvas, 8), 0);
    assert!(ink(&canvas, 16) > 10);
}

#[test]
fn text_scales_up_in_large_cells() {
    let mut canvas = PixelCanvas::new(2, 1, (16, 32));
    canvas.text(0, 0, "#", RED);
    let drawn = (0..32)
        .flat_map(|y| (0..16).map(move |x| (x, y)))
        .filter(|&(x, y)| canvas.get(x, y).is_some())
        .count();
    // At scale 2 every font pixel covers four device pixels.
    let mut reference = PixelCanvas::new(2, 1, (8, 8));
    reference.text(0, 0, "#", RED);
    let base = (0..8)
        .flat_map(|y| (0..8).map(move |x| (x, y)))
        .filter(|&(x, y)| reference.get(x, y).is_some())
        .count();
    assert_eq!(drawn, base * 4);
}
