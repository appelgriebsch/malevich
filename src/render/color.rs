//! Colors and color modes, with honest downhill quantization.
//!
//! A [`Color`] names intent (a palette entry, a 256-index, or exact RGB); a
//! [`ColorMode`] names what the output may carry. Encoding resolves every color to
//! the mode's tier: RGB quantizes to the 256-color cube, 256-indices quantize to the
//! nearest of the 16 ANSI colors, and in [`ColorMode::Plain`] color vanishes
//! entirely. The 16 named colors are never upconverted — they stay palette-relative,
//! so terminal themes keep deciding what they look like.

/// How much color the output may carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ColorMode {
    /// No escape codes at all: safe for files, pipes, and logs.
    Plain,
    /// The 16-color ANSI palette.
    Ansi16,
    /// The xterm 256-color palette.
    Ansi256,
    /// 24-bit RGB.
    TrueColor,
}

/// A terminal color.
///
/// The named variants map to SGR codes 30–37 and 90–97; what they look like is the
/// terminal theme's decision — which is what makes them safe defaults.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(missing_docs)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Color {
    #[default]
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    /// An index into the xterm 256-color palette.
    Ansi256(u8),
    /// An exact 24-bit color.
    Rgb(u8, u8, u8),
}

/// A color resolved against a mode: what will actually be emitted.
///
/// Equality on resolved colors drives run-length encoding, so two colors that
/// quantize identically share one escape sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Resolved {
    Default,
    /// A complete SGR code for one of the 16 palette colors (30–37, 90–97).
    Indexed16(u8),
    Indexed256(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    /// Resolves this color to what `mode` can carry.
    pub(crate) fn resolve(self, mode: ColorMode) -> Resolved {
        if mode == ColorMode::Plain {
            return Resolved::Default;
        }
        match self {
            Color::Default => Resolved::Default,
            Color::Ansi256(index) => match mode {
                ColorMode::Ansi16 => {
                    let (r, g, b) = ansi256_to_rgb(index);
                    Resolved::Indexed16(rgb_to_16(r, g, b))
                }
                _ => Resolved::Indexed256(index),
            },
            Color::Rgb(r, g, b) => match mode {
                ColorMode::TrueColor => Resolved::Rgb(r, g, b),
                ColorMode::Ansi256 => Resolved::Indexed256(rgb_to_256(r, g, b)),
                _ => Resolved::Indexed16(rgb_to_16(r, g, b)),
            },
            named => Resolved::Indexed16(named.sgr()),
        }
    }

    /// The concrete RGB this color denotes in output that cannot stay
    /// palette-relative: named colors freeze to the xterm defaults the quantizer
    /// already assumes, `Default` to a mid-gray readable on dark and light
    /// backgrounds alike.
    pub(crate) fn to_rgb(self) -> (u8, u8, u8) {
        match self {
            Color::Default => (128, 128, 128),
            Color::Ansi256(index) => ansi256_to_rgb(index),
            Color::Rgb(r, g, b) => (r, g, b),
            named => {
                let sgr = named.sgr();
                let offset = if sgr >= 90 { sgr - 90 + 8 } else { sgr - 30 };
                PALETTE16[offset as usize]
            }
        }
    }

    /// The SGR foreground code for a named color.
    fn sgr(self) -> u8 {
        match self {
            Color::Black => 30,
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::White => 37,
            Color::BrightBlack => 90,
            Color::BrightRed => 91,
            Color::BrightGreen => 92,
            Color::BrightYellow => 93,
            Color::BrightBlue => 94,
            Color::BrightMagenta => 95,
            Color::BrightCyan => 96,
            Color::BrightWhite => 97,
            _ => 39,
        }
    }
}

impl Resolved {
    /// Appends one SGR sequence for the foreground and/or background channels
    /// that changed. Keeping both parameters in one control sequence avoids
    /// doubling terminal transitions for two-color cells.
    pub(crate) fn write_transition(
        foreground: Option<Resolved>,
        background: Option<Resolved>,
        out: &mut String,
    ) {
        if foreground.is_none() && background.is_none() {
            return;
        }
        out.push_str("\x1b[");
        if let Some(color) = foreground {
            color.write_parameters(out, false);
            if background.is_some() {
                out.push(';');
            }
        }
        if let Some(color) = background {
            color.write_parameters(out, true);
        }
        out.push('m');
    }

    fn write_parameters(self, out: &mut String, background: bool) {
        use std::fmt::Write as _;
        let _ = match self {
            Resolved::Default => write!(out, "{}", if background { 49 } else { 39 }),
            Resolved::Indexed16(code) => {
                write!(out, "{}", if background { code + 10 } else { code })
            }
            Resolved::Indexed256(index) => {
                write!(out, "{};5;{index}", if background { 48 } else { 38 })
            }
            Resolved::Rgb(r, g, b) => {
                write!(out, "{};2;{r};{g};{b}", if background { 48 } else { 38 })
            }
        };
    }
}

/// The xterm default RGB values of the 16 palette colors, in SGR order
/// (30–37 then 90–97). Used only for quantization distance — the terminal's real
/// palette may differ, which is the point of named colors.
const PALETTE16: [(u8, u8, u8); 16] = [
    (0, 0, 0),
    (205, 0, 0),
    (0, 205, 0),
    (205, 205, 0),
    (0, 0, 238),
    (205, 0, 205),
    (0, 205, 205),
    (229, 229, 229),
    (127, 127, 127),
    (255, 0, 0),
    (0, 255, 0),
    (255, 255, 0),
    (92, 92, 255),
    (255, 0, 255),
    (0, 255, 255),
    (255, 255, 255),
];

/// The cube axis levels of the xterm 256-color palette.
const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// Quantizes RGB onto the xterm 256-color palette (color cube or gray ramp).
pub(crate) fn rgb_to_256(r: u8, g: u8, b: u8) -> u8 {
    if r == g && g == b {
        // The nearest of the 24 ramp greys (8, 18, …, 238) and the cube's two
        // ends (0 and 255): every level from 0 to 255 has one, so a near-black
        // grey lands on the darkest ramp entry instead of underflowing past it.
        let level = i32::from(r);
        let step = ((level - 8 + 5).div_euclid(10)).clamp(0, 23);
        let ramp = (232 + step) as u8;
        let ramp_level = 8 + 10 * step;
        let candidates = [(16u8, 0), (231, 255), (ramp, ramp_level)];
        return candidates
            .into_iter()
            .min_by_key(|&(_, value)| ((value - level).abs(), value != ramp_level))
            .map_or(ramp, |(index, _)| index);
    }
    let axis = |c: u8| match c {
        0..=47 => 0u8,
        48..=114 => 1,
        c => (c - 35) / 40,
    };
    16 + 36 * axis(r) + 6 * axis(g) + axis(b)
}

/// The RGB value of an xterm 256-color palette index.
pub(crate) fn ansi256_to_rgb(index: u8) -> (u8, u8, u8) {
    match index {
        0..=15 => PALETTE16[index as usize],
        16..=231 => {
            let n = index - 16;
            (
                CUBE[(n / 36) as usize],
                CUBE[((n / 6) % 6) as usize],
                CUBE[(n % 6) as usize],
            )
        }
        gray => {
            let v = 8 + 10 * (gray - 232);
            (v, v, v)
        }
    }
}

/// sRGB to OKLab (Björn Ottosson, 2020): the perceptual space the crate
/// mixes and compares colors in. A straight line between two stops keeps
/// its lightness and hue honest — no grey mud halfway between blue and
/// yellow — and the nearest of sixteen palette colors is the one that
/// looks nearest, not the one closest in RGB.
pub(crate) fn oklab((r, g, b): (u8, u8, u8)) -> (f64, f64, f64) {
    let linear = |channel: u8| {
        let c = f64::from(channel) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let (r, g, b) = (linear(r), linear(g), linear(b));
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    (
        0.210_454_255_3 * l + 0.793_617_785 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766 * s,
    )
}

/// OKLab back to sRGB, clamped into the gamut.
pub(crate) fn from_oklab((lightness, a, b): (f64, f64, f64)) -> (u8, u8, u8) {
    let l = (lightness + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
    let m = (lightness - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
    let s = (lightness - 0.089_484_177_5 * a - 1.291_485_548 * b).powi(3);
    let gamma = |c: f64| {
        let c = if c.is_finite() {
            c.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let encoded = if c <= 0.003_130_8 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (encoded * 255.0).round() as u8
    };
    (
        gamma(4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s),
        gamma(-1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s),
        gamma(-0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701 * s),
    )
}

/// The OKLab chroma below which a color reads as a grey. A color with
/// visible chroma keeps a hue when it drops to sixteen colors — a teal
/// becomes green, never the grey that is nearer in distance — and a tint
/// too faint to read as colored goes to the greys.
const ACHROMATIC_CHROMA: f64 = 0.03;

/// The chroma every chromatic color is compared at: sixteen colors carry a
/// hue and a lightness, so the pick matches those and sets chroma aside.
const REFERENCE_CHROMA: f64 = 0.15;

/// Quantizes RGB to the nearest of the 16 palette colors in OKLab —
/// lightness and hue, at one reference chroma, within the color's own
/// class of chromatic or grey — returning its SGR code.
pub(crate) fn rgb_to_16(r: u8, g: u8, b: u8) -> u8 {
    let chroma = |(_, a, b): (f64, f64, f64)| a.hypot(b);
    // Lightness, and the hue as a point on a circle of reference chroma.
    let place = |lab: (f64, f64, f64)| {
        let c = chroma(lab);
        if c > ACHROMATIC_CHROMA {
            (
                lab.0,
                lab.1 / c * REFERENCE_CHROMA,
                lab.2 / c * REFERENCE_CHROMA,
            )
        } else {
            (lab.0, 0.0, 0.0)
        }
    };
    let target = oklab((r, g, b));
    let colored = chroma(target) > ACHROMATIC_CHROMA;
    let target = place(target);
    let mut best = 0usize;
    let mut best_distance = f64::INFINITY;
    for (index, &entry) in PALETTE16.iter().enumerate() {
        let candidate = oklab(entry);
        if (chroma(candidate) > ACHROMATIC_CHROMA) != colored {
            continue;
        }
        let candidate = place(candidate);
        let distance = (target.0 - candidate.0).powi(2)
            + (target.1 - candidate.1).powi(2)
            + (target.2 - candidate.2).powi(2);
        if distance < best_distance {
            best_distance = distance;
            best = index;
        }
    }
    if best < 8 {
        30 + best as u8
    } else {
        90 + (best as u8 - 8)
    }
}

#[cfg(test)]
#[path = "tests/color_tests.rs"]
mod tests;
