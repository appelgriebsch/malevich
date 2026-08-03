use super::{adler32, crc32, encode};
use crate::pixel::render::Image;

fn image(width: usize, height: usize, pixels: Vec<Option<(u8, u8, u8)>>) -> Image {
    Image {
        width,
        height,
        pixels,
    }
}

#[test]
fn crc32_matches_the_classic_check_value() {
    assert_eq!(crc32(b"123456789".iter().copied()), 0xCBF4_3926);
    assert_eq!(crc32(b"".iter().copied()), 0);
}

#[test]
fn adler32_matches_known_values() {
    assert_eq!(adler32(b""), 1);
    assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
}

#[test]
fn the_png_container_is_structurally_sound() {
    let png = encode(&image(
        2,
        2,
        vec![Some((255, 0, 0)), None, None, Some((0, 0, 255))],
    ));
    // Signature.
    assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    // IHDR: length 13, width 2, height 2, depth 8, color type 6 (RGBA).
    assert_eq!(&png[8..12], &13u32.to_be_bytes());
    assert_eq!(&png[12..16], b"IHDR");
    assert_eq!(&png[16..20], &2u32.to_be_bytes());
    assert_eq!(&png[20..24], &2u32.to_be_bytes());
    assert_eq!(&png[24..26], &[8, 6]);
    // The file ends with IEND and its fixed CRC.
    assert_eq!(
        &png[png.len() - 12..],
        &[0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xAE, 0x42, 0x60, 0x82]
    );
}

#[test]
fn the_idat_stream_inflates_to_the_scanlines() {
    let png = encode(&image(2, 1, vec![Some((10, 20, 30)), None]));
    // Find IDAT and hand-inflate its stored block.
    let idat = png
        .windows(4)
        .position(|w| w == b"IDAT")
        .expect("an IDAT chunk exists");
    let length = u32::from_be_bytes(png[idat - 4..idat].try_into().unwrap()) as usize;
    let data = &png[idat + 4..idat + 4 + length];
    // zlib header, one final stored block: 1, LEN, NLEN, then the raw bytes.
    assert_eq!(&data[..2], &[0x78, 0x01]);
    assert_eq!(data[2], 1);
    let raw_len = u16::from_le_bytes([data[3], data[4]]) as usize;
    assert_eq!(u16::from_le_bytes([data[5], data[6]]), !(raw_len as u16));
    let raw = &data[7..7 + raw_len];
    // One scanline: filter 0, RGBA(10, 20, 30, 255), transparent RGBA(0,0,0,0).
    assert_eq!(raw, &[0, 10, 20, 30, 255, 0, 0, 0, 0]);
    // The trailer is the adler of exactly those bytes.
    let trailer = u32::from_be_bytes(data[7 + raw_len..7 + raw_len + 4].try_into().unwrap());
    assert_eq!(trailer, adler32(raw));
}

#[test]
fn large_images_split_into_multiple_stored_blocks() {
    // 200×90 RGBA = 72,180 raw bytes with filters — two stored blocks.
    let png = encode(&image(200, 90, vec![Some((1, 2, 3)); 18000]));
    let idat = png
        .windows(4)
        .position(|w| w == b"IDAT")
        .expect("an IDAT chunk exists");
    let data = &png[idat + 4..];
    // First block is not final (0), second is (1): byte 2 of the stream is the
    // first block header.
    assert_eq!(data[2], 0);
    let first_len = u16::from_le_bytes([data[3], data[4]]) as usize;
    assert_eq!(first_len, 65535);
    assert_eq!(data[7 + first_len], 1);
}
