//! The sixel encoder: palette-indexed bands of six vertical pixels (DEC, 1987).
//!
//! Emitted with `P2 = 1`, so pixels nothing drew keep the terminal background —
//! transparency by omission, no background guessing. The palette is the sorted
//! set of distinct panel colors, so register assignment is deterministic; past
//! sixel's 256 registers, colors quantize through the xterm 256-color cube
//! first. Plots rarely get there: a chart is low-color content, and even a
//! truecolor colormap ramp lands within a few hundred distinct values.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::render::{Image, Rgb};
use crate::render::{ansi256_to_rgb, rgb_to_256};

/// Encodes an image; `None` pixels stay transparent.
pub(crate) fn encode(image: &Image) -> String {
    let (width, height) = (image.width, image.height);
    let mut palette = distinct(&image.pixels);
    let quantized: Vec<Option<Rgb>>;
    let pixels = if palette.len() > 256 {
        quantized = image
            .pixels
            .iter()
            .map(|pixel| pixel.map(|(r, g, b)| ansi256_to_rgb(rgb_to_256(r, g, b))))
            .collect();
        palette = distinct(&quantized);
        &quantized[..]
    } else {
        &image.pixels[..]
    };
    let register = |rgb: Rgb| -> u16 {
        palette
            .binary_search(&rgb)
            .expect("every emitted color is in the palette") as u16
    };

    let mut out = String::new();
    // DCS q: P1 is superseded by the 1:1 raster attributes below; P2 = 1 keeps
    // undrawn pixels at the background.
    let _ = write!(out, "\x1bP0;1;0q\"1;1;{width};{height}");
    for (index, (r, g, b)) in palette.iter().enumerate() {
        // Registers take RGB as 0–100 percentages.
        let percent = |c: u8| (u16::from(c) * 100 + 127) / 255;
        let _ = write!(
            out,
            "#{index};2;{};{};{}",
            percent(*r),
            percent(*g),
            percent(*b)
        );
    }

    // Bands of six rows; within a band, one pass per color present, columns as
    // bitmasks (bit 0 the band's top row), `$` returning to the band start
    // between passes, `-` advancing to the next band.
    for band_top in (0..height).step_by(6) {
        let rows = (height - band_top).min(6);
        let mut planes: BTreeMap<u16, Vec<u8>> = BTreeMap::new();
        for dy in 0..rows {
            let row = band_top + dy;
            for x in 0..width {
                if let Some(rgb) = pixels[row * width + x] {
                    planes
                        .entry(register(rgb))
                        .or_insert_with(|| vec![0u8; width])[x] |= 1 << dy;
                }
            }
        }
        let mut first = true;
        for (index, plane) in &planes {
            if !first {
                out.push('$');
            }
            first = false;
            let _ = write!(out, "#{index}");
            let mut start = 0;
            while start < width {
                let byte = plane[start];
                let mut end = start + 1;
                while end < width && plane[end] == byte {
                    end += 1;
                }
                let count = end - start;
                let glyph = char::from(byte + 0x3F);
                // `!n` run-length pays for itself past three repeats.
                if count >= 4 {
                    let _ = write!(out, "!{count}{glyph}");
                } else {
                    for _ in 0..count {
                        out.push(glyph);
                    }
                }
                start = end;
            }
        }
        if band_top + 6 < height {
            out.push('-');
        }
    }
    out.push_str("\x1b\\");
    out
}

/// The distinct colors of the image, sorted.
fn distinct(pixels: &[Option<Rgb>]) -> Vec<Rgb> {
    let mut colors: Vec<Rgb> = pixels.iter().flatten().copied().collect();
    colors.sort_unstable();
    colors.dedup();
    colors
}

#[cfg(test)]
#[path = "tests/sixel_tests.rs"]
mod tests;
