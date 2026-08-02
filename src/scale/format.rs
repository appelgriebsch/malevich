//! Exact decimal formatting for tick labels.
//!
//! Tick values are represented as `mantissa * 10^exp10` with an integer mantissa, so
//! labels are produced by integer math alone: no binary-float artifacts (`0.30000...4`),
//! no `-0`, and a uniform number of decimals across an axis.

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

#[cfg(test)]
#[path = "tests/format_tests.rs"]
mod tests;
