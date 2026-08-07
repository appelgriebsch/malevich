use super::encode;
use crate::pixel::render::Image;

const RED: Option<(u8, u8, u8)> = Some((255, 0, 0));
const BLUE: Option<(u8, u8, u8)> = Some((0, 0, 255));

fn image(width: usize, height: usize, pixels: Vec<Option<(u8, u8, u8)>>) -> Image {
    Image {
        width,
        height,
        pixels,
    }
}

#[test]
fn a_solid_image_encodes_to_one_register_and_one_band() {
    let out = encode(&image(2, 2, vec![RED, RED, RED, RED]));
    // Header: transparent background (P2=1), 1:1 raster attributes, size 2×2.
    // One register (100;0;0 percent), both columns carrying rows 0 and 1
    // (bits 0b11 → 0x42, 'B'), no band separator, string terminator.
    assert_eq!(out, "\x1bP0;1;0q\"1;1;2;2#0;2;100;0;0#0BB\x1b\\");
}

#[test]
fn transparent_pixels_are_simply_not_emitted() {
    let out = encode(&image(2, 2, vec![RED, None, None, None]));
    // Only (0, 0) is set: bit 0 in column 0 ('@'), column 1 empty ('?').
    assert_eq!(out, "\x1bP0;1;0q\"1;1;2;2#0;2;100;0;0#0@?\x1b\\");
}

#[test]
fn two_colors_share_a_band_via_carriage_return() {
    let out = encode(&image(2, 1, vec![RED, BLUE]));
    // Registers sort by RGB: blue first. Each pass covers one column.
    assert_eq!(
        out,
        "\x1bP0;1;0q\"1;1;2;1#0;2;0;0;100#1;2;100;0;0#0?@$#1@?\x1b\\"
    );
}

#[test]
fn long_runs_use_repeat_introducers() {
    let pixels = vec![RED; 100];
    let out = encode(&image(100, 1, pixels.clone()));
    assert!(out.contains("!100@"), "{out}");
}

#[test]
fn tall_images_split_into_six_row_bands() {
    let pixels = vec![RED; 7];
    let out = encode(&image(1, 7, pixels.clone()));
    // Rows 0–5 fill the first band (0b111111 → '~'), row 6 starts the second.
    assert!(out.contains("#0~-#0@"), "{out}");
}

#[test]
fn more_than_256_colors_quantize_into_the_terminal_cube() {
    // 400 distinct colors: a smooth ramp.
    let pixels: Vec<Option<(u8, u8, u8)>> = (0..400)
        .map(|i| Some(((i % 200) as u8, (i / 2) as u8, 7)))
        .collect();
    let out = encode(&image(400, 1, pixels.clone()));
    let registers = out.matches(";2;").count();
    assert!(registers <= 256, "palette overflow: {registers} registers");
}

#[test]
fn encoding_is_deterministic() {
    let pixels = vec![RED, BLUE, None, RED, BLUE, None];
    assert_eq!(
        encode(&image(3, 2, pixels.clone())),
        encode(&image(3, 2, pixels.clone()))
    );
}

#[test]
fn every_tiny_raster_encodes_deterministically() {
    for width in 0..=4 {
        for height in 0..=4 {
            let pixels = (0..width * height)
                .map(|index| match index % 3 {
                    0 => RED,
                    1 => None,
                    _ => BLUE,
                })
                .collect();
            let image = image(width, height, pixels);
            let first = encode(&image);
            assert_eq!(first, encode(&image));
            assert!(first.starts_with("\x1bP0;1;0q"));
            assert!(first.ends_with("\x1b\\"));
        }
    }
}
