use super::{cumsum, diff, normalize, rank};
use crate::stat::Reducer;

#[test]
fn running_sums_skip_gaps_without_absorbing_them() {
    let sums = cumsum(&[1.0, f64::NAN, 2.0, 3.0]);
    assert_eq!(sums[0], 1.0);
    assert!(sums[1].is_nan());
    assert_eq!(&sums[2..], [3.0, 6.0]);
    assert!(cumsum(&[]).is_empty());
}

#[test]
fn differences_gap_beside_gaps_and_at_the_start() {
    let deltas = diff(&[1.0, 4.0, f64::NAN, 9.0, 10.0]);
    assert!(deltas[0].is_nan());
    assert_eq!(deltas[1], 3.0);
    assert!(deltas[2].is_nan());
    assert!(deltas[3].is_nan());
    assert_eq!(deltas[4], 1.0);
}

#[test]
fn ranks_are_ascending_zero_based_and_tie_low() {
    assert_eq!(rank(&[30.0, 10.0, 20.0, 10.0]), [3.0, 0.0, 2.0, 0.0]);
    let with_gap = rank(&[2.0, f64::NAN, 1.0]);
    assert_eq!(with_gap[0], 1.0);
    assert!(with_gap[1].is_nan());
    assert_eq!(with_gap[2], 0.0);
}

#[test]
fn normalization_divides_by_a_reduction_and_refuses_a_zero_basis() {
    assert_eq!(normalize(&[2.0, 4.0, 1.0], Reducer::Max), [0.5, 1.0, 0.25]);
    assert_eq!(normalize(&[1.0, 3.0], Reducer::Sum), [0.25, 0.75]);
    assert_eq!(normalize(&[50.0, 60.0], Reducer::First), [1.0, 1.2]);
    assert!(
        normalize(&[0.0, 0.0], Reducer::Max)
            .iter()
            .all(|v| v.is_nan())
    );
    assert!(normalize(&[], Reducer::Mean).is_empty());
    assert!(normalize(&[1.0], Reducer::Deviation)[0].is_nan());
}
