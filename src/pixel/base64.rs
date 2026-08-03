//! Standard base64 (RFC 4648, with padding), hand-rolled: two small consumers
//! (kitty, iTerm2) do not justify a dependency.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(crate) fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = u32::from_be_bytes([
            0,
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ]);
        out.push(char::from(ALPHABET[(n >> 18 & 63) as usize]));
        out.push(char::from(ALPHABET[(n >> 12 & 63) as usize]));
        out.push(if chunk.len() > 1 {
            char::from(ALPHABET[(n >> 6 & 63) as usize])
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            char::from(ALPHABET[(n & 63) as usize])
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
#[path = "tests/base64_tests.rs"]
mod tests;
