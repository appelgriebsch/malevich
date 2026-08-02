use super::ecdf;

#[test]
fn fractions_climb_from_one_nth_to_one() {
    let (x, y) = ecdf(&[3.0, 1.0, 2.0, 4.0]);
    assert_eq!(x, [1.0, 2.0, 3.0, 4.0]);
    assert_eq!(y, [0.25, 0.5, 0.75, 1.0]);
}

#[test]
fn gaps_are_excluded_from_the_distribution() {
    let (x, y) = ecdf(&[2.0, f64::NAN, 1.0]);
    assert_eq!(x, [1.0, 2.0]);
    assert_eq!(y, [0.5, 1.0]);
}

#[test]
fn nothing_finite_yields_an_empty_distribution() {
    let (x, y) = ecdf(&[f64::NAN]);
    assert!(x.is_empty() && y.is_empty());
}
