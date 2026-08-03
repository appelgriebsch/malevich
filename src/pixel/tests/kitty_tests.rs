use super::encode;
use crate::pixel::render::Image;

fn image(width: usize, height: usize, pixels: Vec<Option<(u8, u8, u8)>>) -> Image {
    Image {
        width,
        height,
        pixels,
    }
}

#[test]
fn a_single_pixel_encodes_to_one_complete_apc_escape() {
    let out = encode(&image(1, 1, vec![Some((255, 0, 0))]));
    // RGBA (255, 0, 0, 255) → base64 "/wAA/w==".
    assert_eq!(out, "\x1b_Ga=T,f=32,s=1,v=1,C=1,q=2,m=0;/wAA/w==\x1b\\");
}

#[test]
fn transparent_pixels_carry_zero_alpha() {
    let out = encode(&image(1, 1, vec![None]));
    assert_eq!(out, "\x1b_Ga=T,f=32,s=1,v=1,C=1,q=2,m=0;AAAAAA==\x1b\\");
}

#[test]
fn large_images_chunk_at_4096_bytes_of_payload() {
    // 2048 pixels → 8192 RGBA bytes → 10924 base64 chars → 3 chunks.
    let out = encode(&image(64, 32, vec![Some((1, 2, 3)); 2048]));
    let escapes = out.matches("\x1b_G").count();
    assert_eq!(escapes, 3);
    // Control keys only on the first chunk; continuation flags on all: two
    // more-to-come chunks, one final.
    assert_eq!(out.matches("a=T").count(), 1);
    assert_eq!(out.matches("m=1;").count(), 2);
    assert_eq!(out.matches("m=0;").count(), 1);
    // Every chunk terminates properly.
    assert_eq!(out.matches("\x1b\\").count(), 3);
}

#[test]
fn encoding_is_deterministic() {
    let pixels = vec![Some((9, 8, 7)), None, Some((1, 2, 3)), None];
    assert_eq!(
        encode(&image(2, 2, pixels.clone())),
        encode(&image(2, 2, pixels))
    );
}
