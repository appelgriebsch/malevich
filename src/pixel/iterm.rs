//! The iTerm2 inline-image encoder: a PNG inside an OSC 1337 escape.
//!
//! `width`/`height` are given in cells, pinning the image to the exact cell box
//! the layout reserved — the terminal scales if its real cell geometry differs
//! from the configured one, so chrome and panel can never drift apart.
//! `doNotMoveCursor=1` leaves the cursor where the image began: the renderer
//! restores its saved position afterwards anyway, and an image that ended at
//! the screen's bottom row would otherwise scroll the screen first and put
//! that restored position one row off.

use super::{base64, png};

pub(crate) fn encode_rgba(
    width: usize,
    height: usize,
    columns: usize,
    rows: usize,
    rgba: &[u8],
) -> String {
    let png = png::encode(width, height, rgba);
    let payload = base64::encode(&png);
    format!(
        "\x1b]1337;File=inline=1;size={};width={columns};height={rows};preserveAspectRatio=0;doNotMoveCursor=1:{payload}\x07",
        png.len()
    )
}
