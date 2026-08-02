use super::Charset;

#[test]
fn densities_match_the_glyph_grids() {
    assert_eq!(Charset::Ascii.pixels_per_cell(), (1, 1));
    assert_eq!(Charset::Braille.pixels_per_cell(), (2, 4));
}

#[test]
fn braille_bits_follow_the_unicode_dot_layout() {
    // Left column, top to bottom: dots 1, 2, 3, 7.
    assert_eq!(Charset::Braille.bit(0, 0), 0x01);
    assert_eq!(Charset::Braille.bit(0, 1), 0x02);
    assert_eq!(Charset::Braille.bit(0, 2), 0x04);
    assert_eq!(Charset::Braille.bit(0, 3), 0x40);
    // Right column, top to bottom: dots 4, 5, 6, 8.
    assert_eq!(Charset::Braille.bit(1, 0), 0x08);
    assert_eq!(Charset::Braille.bit(1, 1), 0x10);
    assert_eq!(Charset::Braille.bit(1, 2), 0x20);
    assert_eq!(Charset::Braille.bit(1, 3), 0x80);
}

#[test]
fn empty_patterns_are_spaces_in_every_charset() {
    assert_eq!(Charset::Ascii.glyph(0), ' ');
    assert_eq!(Charset::Braille.glyph(0), ' ');
}

#[test]
fn braille_glyphs_offset_into_the_braille_block() {
    assert_eq!(Charset::Braille.glyph(0x01), '\u{2801}');
    assert_eq!(Charset::Braille.glyph(0xFF), '\u{28FF}');
}

#[test]
fn ascii_draws_any_lit_pattern_as_a_star() {
    assert_eq!(Charset::Ascii.bit(0, 0), 1);
    assert_eq!(Charset::Ascii.glyph(1), '*');
}
