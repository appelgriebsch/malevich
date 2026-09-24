use super::{Color, ColorMode, Resolved, ansi256_to_rgb, rgb_to_16, rgb_to_256};

#[test]
fn the_cube_corners_quantize_exactly() {
    assert_eq!(rgb_to_256(0, 0, 0), 16);
    assert_eq!(rgb_to_256(255, 255, 255), 231);
    assert_eq!(rgb_to_256(95, 135, 175), 16 + 36 + 6 * 2 + 3);
    assert_eq!(rgb_to_256(255, 0, 0), 16 + 36 * 5);
}

#[test]
fn grays_use_the_gray_ramp() {
    assert_eq!(rgb_to_256(128, 128, 128), 232 + 12);
    assert_eq!(rgb_to_256(8, 8, 8), 232);
    assert_eq!(rgb_to_256(2, 2, 2), 16);
    assert_eq!(rgb_to_256(250, 250, 250), 231);
}

#[test]
fn cube_and_ramp_entries_roundtrip_through_their_rgb() {
    for index in [16u8, 67, 123, 196, 231, 232, 244, 255] {
        let (r, g, b) = ansi256_to_rgb(index);
        assert_eq!(rgb_to_256(r, g, b), index, "index {index} -> ({r},{g},{b})");
    }
}

#[test]
fn primaries_map_to_their_bright_ansi_colors() {
    assert_eq!(rgb_to_16(255, 0, 0), 91);
    assert_eq!(rgb_to_16(205, 0, 0), 31);
    assert_eq!(rgb_to_16(0, 0, 0), 30);
    assert_eq!(rgb_to_16(255, 255, 255), 97);
}

#[test]
fn named_colors_stay_palette_relative_at_every_tier() {
    for mode in [ColorMode::Ansi16, ColorMode::Ansi256, ColorMode::TrueColor] {
        assert_eq!(Color::Red.resolve(mode), Resolved::Indexed16(31));
    }
}

#[test]
fn rgb_resolves_downhill_by_mode() {
    let orange = Color::Rgb(255, 165, 0);
    assert_eq!(
        orange.resolve(ColorMode::TrueColor),
        Resolved::Rgb(255, 165, 0)
    );
    assert_eq!(
        orange.resolve(ColorMode::Ansi256),
        Resolved::Indexed256(rgb_to_256(255, 165, 0))
    );
    assert_eq!(
        orange.resolve(ColorMode::Ansi16),
        Resolved::Indexed16(rgb_to_16(255, 165, 0))
    );
    assert_eq!(orange.resolve(ColorMode::Plain), Resolved::Default);
}

#[test]
fn indexed_colors_only_downgrade_for_sixteen_color_output() {
    let index = Color::Ansi256(196);
    assert_eq!(
        index.resolve(ColorMode::TrueColor),
        Resolved::Indexed256(196)
    );
    assert_eq!(index.resolve(ColorMode::Ansi256), Resolved::Indexed256(196));
    assert_eq!(index.resolve(ColorMode::Ansi16), Resolved::Indexed16(91));
}

#[test]
fn resolved_colors_write_combined_foreground_and_background_sgr_forms() {
    let mut out = String::new();
    Resolved::write_transition(
        Some(Resolved::Indexed16(31)),
        Some(Resolved::Indexed16(34)),
        &mut out,
    );
    Resolved::write_transition(
        Some(Resolved::Indexed256(196)),
        Some(Resolved::Indexed256(21)),
        &mut out,
    );
    Resolved::write_transition(
        Some(Resolved::Rgb(1, 2, 3)),
        Some(Resolved::Rgb(4, 5, 6)),
        &mut out,
    );
    Resolved::write_transition(Some(Resolved::Default), Some(Resolved::Default), &mut out);
    assert_eq!(
        out,
        "\x1b[31;44m\x1b[38;5;196;48;5;21m\x1b[38;2;1;2;3;48;2;4;5;6m\x1b[39;49m"
    );
}

#[test]
fn oklab_round_trips_every_grey_and_the_primaries() {
    use super::{from_oklab, oklab};

    for v in 0..=255u8 {
        assert_eq!(from_oklab(oklab((v, v, v))), (v, v, v), "grey {v}");
    }
    for rgb in [
        (255, 0, 0),
        (0, 255, 0),
        (0, 0, 255),
        (255, 165, 0),
        (86, 180, 233),
    ] {
        assert_eq!(from_oklab(oklab(rgb)), rgb, "{rgb:?}");
    }
    // Lightness orders greys, and a saturated color sits off the grey axis.
    assert!(oklab((0, 0, 0)).0 < oklab((128, 128, 128)).0);
    assert!(oklab((128, 128, 128)).0 < oklab((255, 255, 255)).0);
    assert!(oklab((255, 0, 0)).1.abs() > 0.1);
}

#[test]
fn the_sixteen_color_pick_is_perceptual() {
    // A dim orange is nearer to red than to yellow to the eye; RGB distance
    // would call it yellow-ish. Either way it must land on a red or yellow,
    // never on green or blue.
    let code = rgb_to_16(200, 90, 0);
    assert!([31, 33, 91, 93].contains(&code), "{code}");
    // Near-black and near-white stay black and white.
    assert_eq!(rgb_to_16(10, 10, 10), 30);
    assert_eq!(rgb_to_16(245, 245, 245), 97);
}
