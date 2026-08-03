//! A minimal PNG encoder: RGBA8, stored (uncompressed) deflate blocks.
//!
//! The iTerm2 protocol needs a real image format with alpha, and a zlib stream
//! of stored blocks plus two hand-rolled checksums is a fraction of the weight
//! of an encoder dependency. Stored blocks trade bytes for simplicity — a plot
//! panel is small, and the string is transient.

use super::render::Image;

pub(crate) fn encode(image: &Image) -> Vec<u8> {
    // Raw scanlines: filter byte 0 (None), then RGBA per pixel; alpha 0 keeps
    // undrawn panel area transparent.
    let mut raw = Vec::with_capacity(image.height * (1 + image.width * 4));
    for y in 0..image.height {
        raw.push(0);
        for x in 0..image.width {
            match image.pixels[y * image.width + x] {
                Some((r, g, b)) => raw.extend_from_slice(&[r, g, b, 255]),
                None => raw.extend_from_slice(&[0, 0, 0, 0]),
            }
        }
    }
    let mut png = Vec::new();
    png.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(image.width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(image.height as u32).to_be_bytes());
    // Bit depth 8, color type 6 (truecolor with alpha), deflate, no filter,
    // no interlace.
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &zlib_stored(&raw));
    chunk(&mut png, b"IEND", &[]);
    png
}

/// Appends one chunk: length, type, data, CRC-32 over type and data.
fn chunk(png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    png.extend_from_slice(&(data.len() as u32).to_be_bytes());
    png.extend_from_slice(kind);
    png.extend_from_slice(data);
    let crc = crc32(kind.iter().chain(data).copied());
    png.extend_from_slice(&crc.to_be_bytes());
}

/// A zlib stream of stored (BTYPE 00) deflate blocks: two-byte header,
/// ≤ 65535-byte blocks each carrying LEN and its complement, adler-32 trailer.
fn zlib_stored(raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len() + raw.len() / 65535 * 5 + 16);
    out.extend_from_slice(&[0x78, 0x01]);
    if raw.is_empty() {
        // A valid stream still needs one final (empty) block.
        out.extend_from_slice(&[1, 0, 0, 0xFF, 0xFF]);
    }
    let mut blocks = raw.chunks(65535).peekable();
    while let Some(block) = blocks.next() {
        out.push(u8::from(blocks.peek().is_none()));
        let length = block.len() as u16;
        out.extend_from_slice(&length.to_le_bytes());
        out.extend_from_slice(&(!length).to_le_bytes());
        out.extend_from_slice(block);
    }
    out.extend_from_slice(&adler32(raw).to_be_bytes());
    out
}

const CRC_TABLE: [u32; 256] = crc_table();

const fn crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut n = 0;
    while n < 256 {
        let mut c = n as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
            k += 1;
        }
        table[n] = c;
        n += 1;
    }
    table
}

fn crc32(bytes: impl Iterator<Item = u8>) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for byte in bytes {
        crc = CRC_TABLE[((crc ^ u32::from(byte)) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

fn adler32(bytes: &[u8]) -> u32 {
    const MOD: u32 = 65521;
    let (mut a, mut b) = (1u32, 0u32);
    for chunk in bytes.chunks(5552) {
        // 5552 is the classic largest run before the sums can overflow u32.
        for &byte in chunk {
            a += u32::from(byte);
            b += a;
        }
        a %= MOD;
        b %= MOD;
    }
    (b << 16) | a
}

#[cfg(test)]
#[path = "tests/png_tests.rs"]
mod tests;
