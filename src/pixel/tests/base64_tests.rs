use super::encode;

#[test]
fn the_rfc_4648_test_vectors_encode_exactly() {
    // RFC 4648 §10.
    assert_eq!(encode(b""), "");
    assert_eq!(encode(b"f"), "Zg==");
    assert_eq!(encode(b"fo"), "Zm8=");
    assert_eq!(encode(b"foo"), "Zm9v");
    assert_eq!(encode(b"foob"), "Zm9vYg==");
    assert_eq!(encode(b"fooba"), "Zm9vYmE=");
    assert_eq!(encode(b"foobar"), "Zm9vYmFy");
}

#[test]
fn binary_bytes_round_the_full_alphabet() {
    assert_eq!(encode(&[0xFF, 0xFF, 0xFF]), "////");
    assert_eq!(encode(&[0x00, 0x00, 0x00]), "AAAA");
}
