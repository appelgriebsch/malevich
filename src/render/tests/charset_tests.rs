use super::Charset;

#[test]
fn densities_match_the_glyph_grids() {
    assert_eq!(Charset::Ascii.pixels_per_cell(), (1, 1));
    assert_eq!(Charset::HalfBlocks.pixels_per_cell(), (1, 2));
    assert_eq!(Charset::Quadrants.pixels_per_cell(), (2, 2));
    assert_eq!(Charset::Braille.pixels_per_cell(), (2, 4));
}

#[test]
fn half_blocks_cover_their_four_patterns() {
    assert_eq!(Charset::HalfBlocks.glyph(0), ' ');
    assert_eq!(Charset::HalfBlocks.glyph(1), '\u{2580}');
    assert_eq!(Charset::HalfBlocks.glyph(2), '\u{2584}');
    assert_eq!(Charset::HalfBlocks.glyph(3), '\u{2588}');
}

#[test]
fn quadrant_glyphs_match_their_bit_patterns() {
    // Top-left only, both top pixels, the diagonal, and the full cell.
    assert_eq!(Charset::Quadrants.bit(0, 0), 1);
    assert_eq!(Charset::Quadrants.bit(1, 1), 8);
    assert_eq!(Charset::Quadrants.glyph(0b0001), '\u{2598}');
    assert_eq!(Charset::Quadrants.glyph(0b0011), '\u{2580}');
    assert_eq!(Charset::Quadrants.glyph(0b1001), '\u{259A}');
    assert_eq!(Charset::Quadrants.glyph(0b1111), '\u{2588}');
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

#[test]
fn sextant_glyphs_follow_the_legacy_computing_formula() {
    assert_eq!(Charset::Sextants.pixels_per_cell(), (2, 3));
    assert_eq!(Charset::Sextants.glyph(0), ' ');
    // The three legacy exceptions keep their old glyphs.
    assert_eq!(Charset::Sextants.glyph(21), '\u{258C}');
    assert_eq!(Charset::Sextants.glyph(42), '\u{2590}');
    assert_eq!(Charset::Sextants.glyph(63), '\u{2588}');
    // The block starts right after the empty pattern…
    assert_eq!(Charset::Sextants.glyph(1), '\u{1FB00}');
    // …and skips the exceptions as it goes: the last dense pattern is 62.
    assert_eq!(Charset::Sextants.glyph(62), '\u{1FB3B}');
}

#[test]
fn octant_glyphs_reuse_legacy_characters_where_they_exist() {
    assert_eq!(Charset::Octants.pixels_per_cell(), (2, 4));
    assert_eq!(Charset::Octants.glyph(0), ' ');
    assert_eq!(Charset::Octants.glyph(0b0000_0101), '\u{2598}'); // upper-left quadrant
    assert_eq!(Charset::Octants.glyph(0b0000_1111), '\u{2580}'); // upper half
    assert_eq!(Charset::Octants.glyph(0b0101_0101), '\u{258C}'); // left half
    assert_eq!(Charset::Octants.glyph(0b1111_0000), '\u{2584}'); // lower half
    assert_eq!(Charset::Octants.glyph(255), '\u{2588}');
    // A pattern with no legacy twin lands in the Unicode 16 legacy-computing
    // supplement (the octants span its main run plus a few outliers).
    for novel_bits in [
        0b0000_0001u8,
        0b0000_0010,
        0b0100_0000,
        0b1000_0000,
        0b0001_0110,
    ] {
        let novel = Charset::Octants.glyph(novel_bits) as u32;
        assert!((0x1CC00..=0x1CEBF).contains(&novel), "got {novel:#x}");
    }
}
