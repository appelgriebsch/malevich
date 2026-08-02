use super::decimal;

#[test]
fn formats_zero_without_fraction_as_plain_zero() {
    assert_eq!(decimal(0, 0), "0");
    assert_eq!(decimal(0, 3), "0");
}

#[test]
fn formats_zero_with_fraction_digits_for_alignment() {
    assert_eq!(decimal(0, -2), "0.00");
}

#[test]
fn appends_zeros_for_positive_exponents() {
    assert_eq!(decimal(5, 2), "500");
    assert_eq!(decimal(-12, 1), "-120");
    assert_eq!(decimal(7, 0), "7");
}

#[test]
fn places_the_decimal_point_inside_the_mantissa() {
    assert_eq!(decimal(1234, -2), "12.34");
    assert_eq!(decimal(-1234, -3), "-1.234");
}

#[test]
fn pads_small_mantissas_with_leading_fraction_zeros() {
    assert_eq!(decimal(7, -3), "0.007");
    assert_eq!(decimal(-7, -1), "-0.7");
}

#[test]
fn keeps_the_fraction_width_of_the_exponent() {
    assert_eq!(decimal(50, -2), "0.50");
    assert_eq!(decimal(100, -2), "1.00");
}
