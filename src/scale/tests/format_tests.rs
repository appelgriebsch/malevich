use super::{NumberFormat, decimal};

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

#[test]
fn a_set_shares_fraction_digits_at_the_significant_budget() {
    let format = NumberFormat::for_values(&[0.4821, 0.517, 1.104]);
    assert_eq!(format.format(0.4821), "0.482");
    assert_eq!(format.format(0.517), "0.517");
    assert_eq!(format.format(1.104), "1.104");
    assert_eq!(format.format(0.5), "0.500");
}

#[test]
fn whole_number_sets_carry_no_fraction() {
    let format = NumberFormat::for_values(&[1200.0, 800.0]);
    assert_eq!(format.format(1200.0), "1200");
    assert_eq!(format.format(800.0), "800");
    let counts = NumberFormat::for_values(&[7.0, 3.0]);
    assert_eq!(counts.format(7.0), "7");
    let mixed = NumberFormat::for_values(&[7.0, 0.5]);
    assert_eq!(mixed.format(7.0), "7.000");
    assert_eq!(mixed.format(0.5), "0.500");
}

#[test]
fn large_sets_share_one_si_prefix() {
    let format = NumberFormat::for_values(&[125_000.0, 98_500.0]);
    assert_eq!(format.format(125_000.0), "125.0k");
    assert_eq!(format.format(98_500.0), "98.5k");
    assert_eq!(format.format(0.0), "0");
}

#[test]
fn tiny_sets_share_one_si_prefix() {
    let format = NumberFormat::for_values(&[0.000_123_4, 0.000_5]);
    assert_eq!(format.format(0.000_123_4), "123.4\u{00B5}");
    assert_eq!(format.format(0.000_5), "500.0\u{00B5}");
}

#[test]
fn gaps_and_negatives_keep_their_conventions() {
    let format = NumberFormat::for_values(&[-0.211, 0.982]);
    assert_eq!(format.format(-0.211), "-0.2110");
    assert_eq!(format.format(f64::NAN), "\u{2014}");
    assert_eq!(format.format(f64::INFINITY), "\u{2014}");
}

#[test]
fn empty_and_zero_sets_format_whole_numbers() {
    let empty = NumberFormat::for_values(&[]);
    assert_eq!(empty.format(0.0), "0");
    assert_eq!(empty.format(3.7), "4");
    let zeros = NumberFormat::for_values(&[0.0, f64::NAN]);
    assert_eq!(zeros.format(0.0), "0");
}

#[test]
fn unprefixed_zero_aligns_with_its_column() {
    let format = NumberFormat::for_values(&[0.0, 0.482]);
    assert_eq!(format.format(0.0), "0.0000");
    assert_eq!(format.format(0.482), "0.4820");
}

#[test]
fn sets_beyond_the_prefix_table_write_against_one_power_of_ten() {
    let huge = NumberFormat::for_values(&[f64::MAX, -f64::MAX]);
    // Rounding 1.7976… up to 1.798 would parse to infinity, so the label
    // steps back one unit: finite, and at or below the value it names.
    assert_eq!(huge.format(f64::MAX), "1.797e308");
    assert_eq!(huge.format(-f64::MAX), "-1.797e308");
    assert_eq!(huge.format(0.0), "0");
    let quadrillions = NumberFormat::for_values(&[1e15, 5e15]);
    assert_eq!(quadrillions.format(1e15), "1.000e15");
    assert_eq!(quadrillions.format(2e14), "0.200e15");
    let tiny = NumberFormat::for_values(&[2.5e-20]);
    assert_eq!(tiny.format(2.5e-20), "2.500e-20");
    // Every exponent label parses back to the value at its budget.
    for value in [f64::MAX, 1e15, 2.5e-20, -3.14159e40] {
        let label = NumberFormat::for_values(&[value]).format(value);
        let parsed: f64 = label.parse().expect("an exponent label parses");
        assert!(
            (parsed - value).abs() <= 1e-3 * value.abs(),
            "{label} vs {value}"
        );
    }
}

#[test]
fn the_prefix_table_still_ends_at_tera_and_pico() {
    assert_eq!(NumberFormat::for_values(&[9e14]).format(9e14), "900.0T");
    assert_eq!(NumberFormat::for_values(&[2e-12]).format(2e-12), "2.000p");
    assert_eq!(
        NumberFormat::for_values(&[2e-13]).format(2e-13),
        "2.000e-13"
    );
}

#[test]
fn widening_adds_fraction_digits_under_the_same_prefix() {
    let set = NumberFormat::for_values(&[125_000.0]).widened(2);
    assert_eq!(set.format(125_000.0), "125.000k");
    assert_eq!(NumberFormat::for_values(&[-0.0]).format(-0.0), "0");
}
