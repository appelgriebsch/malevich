//! Exact decimal formatting for tick labels, and the shared set formatter.
//!
//! Tick values are represented as `mantissa * 10^exp10` with an integer mantissa, so
//! labels are produced by integer math alone: no binary-float artifacts (`0.30000...4`),
//! no `-0`, and a uniform number of decimals across an axis. [`NumberFormat`] makes
//! the same decisions once for an arbitrary set of values — a table column, a legend
//! readout — instead of for an axis's ticks.

/// Formats `mantissa * 10^exp10` as a plain decimal string.
///
/// For `exp10 >= 0` the result is an integer (zero is `"0"`, never `"0000"`). For
/// `exp10 < 0` the result carries exactly `-exp10` fraction digits, including for zero
/// (`"0.00"`), so that labels sharing an exponent align.
pub(crate) fn decimal(mantissa: i128, exp10: i32) -> String {
    if exp10 >= 0 {
        if mantissa == 0 {
            return "0".to_string();
        }
        let mut s = mantissa.to_string();
        s.extend(std::iter::repeat_n('0', exp10 as usize));
        return s;
    }

    let fraction_digits = exp10.unsigned_abs() as usize;
    let sign = if mantissa < 0 { "-" } else { "" };
    let digits = mantissa.unsigned_abs().to_string();
    let (integer, fraction) = if digits.len() > fraction_digits {
        let split = digits.len() - fraction_digits;
        (digits[..split].to_string(), digits[split..].to_string())
    } else {
        let padding = "0".repeat(fraction_digits - digits.len());
        ("0".to_string(), format!("{padding}{digits}"))
    };
    format!("{sign}{integer}.{fraction}")
}

/// The fixed significant-digit budget a value set is formatted at.
const SIGNIFICANT_DIGITS: i32 = 4;

/// The one SI prefix for a set whose largest magnitude is `10^magnitude`:
/// engaged at ten thousand and up (`k`, `M`, `G`, `T`) or below a thousandth
/// (`µ`, `n`, `p`), as `(shift, suffix)`; `None` inside that band or beyond
/// the table.
pub(crate) fn si_prefix(magnitude: i32) -> Option<(i32, char)> {
    if magnitude < 4 && magnitude > -4 {
        return None;
    }
    let shift = 3 * magnitude.div_euclid(3);
    let suffix = match shift {
        3 => 'k',
        6 => 'M',
        9 => 'G',
        12 => 'T',
        -6 => '\u{00B5}',
        -9 => 'n',
        -12 => 'p',
        _ => return None,
    };
    Some((shift, suffix))
}

/// The shortest decimal that round-trips to `value`, as an integer digit
/// string and its power of ten: `0.1 + 0.2` is `(30000000000000004, -17)`.
/// Rounding those digits to a budget is integer arithmetic, so a label never
/// inherits a binary-float artifact and never passes through a second,
/// cheaper formatter — at any magnitude, including the ends of the range
/// where a power of ten is no longer exact.
fn shortest_digits(value: f64) -> (i128, i32) {
    let text = format!("{:e}", value.abs());
    let (mantissa, exponent) = text.split_once('e').unwrap_or((text.as_str(), "0"));
    let exponent: i32 = exponent.parse().unwrap_or(0);
    let fraction_digits = mantissa
        .find('.')
        .map_or(0, |point| mantissa.len() - point - 1) as i32;
    let digits: i128 = mantissa
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap_or(0);
    (digits, exponent - fraction_digits)
}

/// Whether `digits × 10^power` is a whole number — whether scaling to that
/// power loses nothing.
fn scales_exactly(digits: i128, power: i32) -> bool {
    if power >= 0 {
        return true;
    }
    if power < -38 {
        return digits == 0;
    }
    digits % 10i128.pow(power.unsigned_abs()) == 0
}

/// `digits × 10^power`, rounded half away from zero to an integer, or `None`
/// when the product needs more than an `i128` holds.
fn scale_digits(digits: i128, power: i32) -> Option<i128> {
    if power >= 0 {
        if power > 36 {
            return None;
        }
        digits.checked_mul(10i128.pow(power as u32))
    } else if power < -38 {
        Some(0)
    } else {
        let divisor = 10i128.pow(power.unsigned_abs());
        let quotient = digits / divisor;
        let remainder = digits % divisor;
        Some(if remainder * 2 >= divisor {
            quotient + 1
        } else {
            quotient
        })
    }
}

/// Uniform formatting for a set of related values — a table column, a readout
/// row: the decisions an axis makes once for its ticks, made once for the set.
///
/// [`NumberFormat::for_values`] derives a shared resolution from the set's
/// largest magnitude — a fixed significant-digit budget — and one SI prefix for
/// the whole set (`k`, `M`, `µ`, …), engaged at ten thousand and up or below a
/// thousandth, exactly like an axis — or, beyond the prefix table, one power
/// of ten the whole set is written against (`1.798e308`). [`NumberFormat::format`]
/// then renders any value at that resolution as an exact decimal: every value
/// carries the same number of fraction digits, so a right-aligned column
/// aligns at the decimal point. Non-finite values format as `—`, the gap
/// convention; zero on a prefixed set is bare `0`, like a tick; an unprefixed
/// set of whole numbers keeps whole labels (a count column never reads
/// `7.000`).
///
/// The one value that leaves the column's resolution is the one it would
/// misstate: a finite value the set's resolution rounds to zero, or to a
/// single significant digit that is not exact, is written at its own
/// resolution instead — a mean of `1000` in a column of gigabytes reads
/// `1000`, never a bare `0` that claims exactly nothing or a `0.001G` that is
/// off by half. Alignment yields to honesty for that value alone.
///
/// ```
/// use malevich::scale::NumberFormat;
///
/// let losses = NumberFormat::for_values(&[0.4821, 0.517, 1.104]);
/// assert_eq!(losses.format(0.4821), "0.482");
/// assert_eq!(losses.format(1.104), "1.104");
/// assert_eq!(losses.format(f64::NAN), "—");
///
/// let counts = NumberFormat::for_values(&[125_000.0, 98_500.0]);
/// assert_eq!(counts.format(125_000.0), "125.0k");
///
/// let means = NumberFormat::for_values(&[1000.0, 1.199e9]);
/// assert_eq!(means.format(1.199e9), "1.199G");
/// assert_eq!(means.format(1000.0), "1000");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberFormat {
    /// The SI shift applied before rendering, with its suffix; `None` is unshifted.
    prefix: Option<(i32, char)>,
    /// Fraction digits every rendered value carries.
    fraction: i32,
    /// The power of ten every value is written against when the set lies
    /// beyond the SI table (`1.798e308`); `None` is the plain decimal form.
    exponent: Option<i32>,
}

impl NumberFormat {
    /// Derives the shared resolution and prefix from the finite values of the set.
    ///
    /// An empty or all-gap set formats whole numbers with no prefix.
    pub fn for_values(values: &[f64]) -> NumberFormat {
        let max_abs = values
            .iter()
            .copied()
            .filter(|value| value.is_finite())
            .fold(0.0f64, |max, value| max.max(value.abs()));
        if max_abs == 0.0 {
            return NumberFormat {
                prefix: None,
                fraction: 0,
                exponent: None,
            };
        }
        let magnitude = max_abs.log10().floor() as i32;
        let prefix = si_prefix(magnitude);
        // Beyond the prefix table — past `T`, below `p` — the set is written
        // against one power of ten instead: `1.798e308`, never 309 digits, and
        // never a `Display` fallback that a smaller chart would not use.
        let exponent =
            (prefix.is_none() && (magnitude >= 4 || magnitude <= -4)).then_some(magnitude);
        let shift = prefix.map_or(0, |(shift, _)| shift);
        // A set of whole numbers keeps whole labels — a count column never
        // reads `7.000`. Under a prefix the budget stays: `98.5k` needs it.
        let whole = prefix.is_none()
            && exponent.is_none()
            && values
                .iter()
                .filter(|value| value.is_finite())
                .all(|value| value.fract() == 0.0);
        let fraction = if whole {
            0
        } else if exponent.is_some() {
            SIGNIFICANT_DIGITS - 1
        } else {
            (SIGNIFICANT_DIGITS - 1 - (magnitude - shift)).max(0)
        };
        NumberFormat {
            prefix,
            fraction,
            exponent,
        }
    }

    /// The same format with `extra` more fraction digits — how an endpoint
    /// fallback keeps two distinct bounds distinct without abandoning the
    /// set's prefix or exponent.
    pub(crate) fn widened(mut self, extra: i32) -> NumberFormat {
        self.fraction += extra;
        self
    }

    /// Renders `value` at the set's resolution.
    ///
    /// Non-finite values are `—`. A set beyond the SI table writes every
    /// value against its power of ten (`1.798e308`, `2.500e-20`), still at the
    /// shared budget; a mantissa no integer can hold (only a widened fallback
    /// could ask) collapses to the exponent form at that budget. A finite
    /// value the resolution would misstate — rounded to zero, or to one
    /// inexact significant digit — is written at its own resolution instead.
    pub fn format(&self, value: f64) -> String {
        if !value.is_finite() {
            return "\u{2014}".to_string();
        }
        let shift = self
            .exponent
            .unwrap_or_else(|| self.prefix.map_or(0, |(shift, _)| shift));
        let (digits, exp10) = shortest_digits(value);
        let sign = if value.is_sign_negative() { -1 } else { 1 };
        let power = exp10 + self.fraction - shift;
        let Some(mantissa) = scale_digits(digits, power) else {
            // More integer digits than a mantissa can hold (only a widened
            // fallback far from its set can ask): the value at the budget,
            // against its own power of ten.
            let magnitude = exp10 + digits.to_string().len() as i32 - 1;
            let mantissa =
                scale_digits(digits, exp10 + SIGNIFICANT_DIGITS - 1 - magnitude).unwrap_or(0);
            return exponent_label(sign * mantissa, 1 - SIGNIFICANT_DIGITS, magnitude);
        };
        // The set's resolution would misstate this value: nothing left of it,
        // or one digit that rounding already moved. Its own resolution keeps
        // the budget, so a small statistic never reads as exactly zero.
        if value != 0.0 && (mantissa == 0 || (mantissa < 10 && !scales_exactly(digits, power))) {
            return NumberFormat::for_values(&[value]).format(value);
        }
        let mantissa = sign * mantissa;
        if mantissa == 0 {
            // A prefixed or exponent zero is deliberately bare, like a tick's.
            return if self.prefix.is_some() || self.exponent.is_some() {
                "0".to_string()
            } else {
                decimal(0, -self.fraction)
            };
        }
        if let Some(exponent) = self.exponent {
            return exponent_label(mantissa, -self.fraction, exponent);
        }
        let mut label = decimal(mantissa, -self.fraction);
        if let Some((_, suffix)) = self.prefix {
            label.push(suffix);
        }
        label
    }
}

/// `mantissa × 10^exp10` written against `exponent`: `1.798e308`. A rounded
/// mantissa that would overflow the finite range on parsing (only the very
/// top of it can) steps back one unit, so every label parses to a finite
/// value at or below the one it names.
fn exponent_label(mantissa: i128, exp10: i32, exponent: i32) -> String {
    let label = format!("{}e{exponent}", decimal(mantissa, exp10));
    if label.parse::<f64>().is_ok_and(f64::is_infinite) {
        let stepped = mantissa - mantissa.signum();
        return format!("{}e{exponent}", decimal(stepped, exp10));
    }
    label
}

#[cfg(test)]
#[path = "tests/format_tests.rs"]
mod tests;
