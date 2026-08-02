use super::super::{Charset, Color, ColorMode};
use super::Surface;

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<Surface>();
const _: () = assert_send_sync::<Color>();
const _: () = assert_send_sync::<Charset>();

#[test]
fn a_dot_lights_one_braille_subpixel() {
    let mut surface = Surface::new(2, 1, Charset::Braille);
    surface.dot(0.0, 0.0, Color::Default);
    assert_eq!(surface.to_plain(), "\u{2801}");
}

#[test]
fn a_bottom_row_line_renders_as_lower_dots() {
    let mut surface = Surface::new(2, 1, Charset::Braille);
    surface.line((0.0, 3.0), (3.0, 3.0), Color::Default);
    assert_eq!(surface.to_plain(), "\u{28C0}\u{28C0}");
}

#[test]
fn a_left_column_line_renders_as_a_wall() {
    let mut surface = Surface::new(1, 1, Charset::Braille);
    surface.line((0.0, 0.0), (0.0, 3.0), Color::Default);
    assert_eq!(surface.to_plain(), "\u{2847}");
}

#[test]
fn a_diagonal_descends_across_cells() {
    let mut surface = Surface::new(2, 1, Charset::Braille);
    surface.line((0.0, 3.0), (3.0, 0.0), Color::Default);
    assert_eq!(surface.to_plain(), "\u{2860}\u{280A}");
}

#[test]
fn segments_clip_to_the_surface() {
    let mut surface = Surface::new(2, 1, Charset::Braille);
    surface.line((-100.0, 2.0), (100.0, 2.0), Color::Default);
    assert_eq!(surface.to_plain(), "\u{2824}\u{2824}");
}

#[test]
fn fully_outside_segments_draw_nothing() {
    let mut surface = Surface::new(2, 1, Charset::Braille);
    surface.line((-5.0, -5.0), (-1.0, -1.0), Color::Default);
    surface.set(99, 0, Color::Default);
    surface.set(-1, 0, Color::Default);
    assert_eq!(surface.to_plain(), "");
}

#[test]
fn non_finite_coordinates_draw_nothing() {
    let mut surface = Surface::new(2, 1, Charset::Braille);
    surface.line((f64::NAN, 0.0), (3.0, 3.0), Color::Default);
    surface.dot(f64::INFINITY, 0.0, Color::Default);
    assert_eq!(surface.to_plain(), "");
}

#[test]
fn text_wins_over_pixels_in_shared_cells() {
    let mut surface = Surface::new(3, 1, Charset::Braille);
    surface.line((0.0, 0.0), (5.0, 0.0), Color::Default);
    surface.text(0, 0, "ab", Color::Default);
    assert_eq!(surface.to_plain(), "ab\u{2809}");
}

#[test]
fn text_clips_at_the_edges() {
    let mut surface = Surface::new(3, 1, Charset::Ascii);
    surface.text(-1, 0, "abcde", Color::Default);
    assert_eq!(surface.to_plain(), "bcd");
    let mut surface = Surface::new(3, 1, Charset::Ascii);
    surface.text(0, -1, "abc", Color::Default);
    surface.text(0, 1, "abc", Color::Default);
    assert_eq!(surface.to_plain(), "");
}

#[test]
fn plain_output_trims_trailing_spaces_and_keeps_leading_ones() {
    let mut surface = Surface::new(4, 2, Charset::Ascii);
    surface.set(0, 0, Color::Default);
    surface.set(1, 1, Color::Default);
    assert_eq!(surface.to_plain(), "*\n *");
}

#[test]
fn empty_surfaces_encode_to_nothing() {
    assert_eq!(Surface::new(0, 0, Charset::Braille).to_plain(), "");
    assert_eq!(
        Surface::new(0, 0, Charset::Braille).encode(ColorMode::Ansi16),
        ""
    );
    assert_eq!(
        Surface::new(3, 0, Charset::Ascii).encode(ColorMode::Ansi16),
        ""
    );
}

#[test]
fn uncolored_surfaces_encode_identically_in_both_encoders() {
    let mut surface = Surface::new(4, 2, Charset::Braille);
    surface.line((0.0, 0.0), (7.0, 7.0), Color::Default);
    assert_eq!(surface.encode(ColorMode::Ansi16), surface.to_plain());
}

#[test]
fn ansi_runs_emit_one_code_per_color_change() {
    let mut surface = Surface::new(3, 1, Charset::Ascii);
    surface.set(0, 0, Color::Red);
    surface.set(1, 0, Color::Red);
    surface.set(2, 0, Color::Blue);
    assert_eq!(
        surface.encode(ColorMode::Ansi16),
        "\x1b[31m**\x1b[34m*\x1b[0m"
    );
}

#[test]
fn the_last_write_owns_a_shared_cell_color() {
    let mut surface = Surface::new(2, 1, Charset::Ascii);
    surface.set(0, 0, Color::Red);
    surface.set(1, 0, Color::Red);
    surface.set(1, 0, Color::Blue);
    assert_eq!(
        surface.encode(ColorMode::Ansi16),
        "\x1b[31m*\x1b[34m*\x1b[0m"
    );
}

#[test]
fn colored_rows_end_with_a_reset_even_after_trimming() {
    let mut surface = Surface::new(4, 1, Charset::Ascii);
    surface.set(0, 0, Color::Green);
    let encoded = surface.encode(ColorMode::Ansi16);
    assert_eq!(encoded, "\x1b[32m*\x1b[0m");
}
