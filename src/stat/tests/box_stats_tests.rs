use super::BoxStats;

#[test]
fn the_five_numbers_of_a_simple_sample() {
    let stats = BoxStats::of(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
    assert_eq!(stats.q1, 2.0);
    assert_eq!(stats.median, 3.0);
    assert_eq!(stats.q3, 4.0);
    assert_eq!(stats.whisker_low, 1.0);
    assert_eq!(stats.whisker_high, 5.0);
    assert!(stats.outliers.is_empty());
}

#[test]
fn type_seven_quantiles_interpolate() {
    let stats = BoxStats::of(&[1.0, 2.0, 3.0, 4.0]).unwrap();
    assert_eq!(stats.q1, 1.75);
    assert_eq!(stats.median, 2.5);
    assert_eq!(stats.q3, 3.25);
}

#[test]
fn far_values_become_outliers_and_whiskers_pull_in() {
    let stats = BoxStats::of(&[1.0, 2.0, 3.0, 4.0, 5.0, 100.0]).unwrap();
    assert_eq!(stats.outliers, [100.0]);
    assert_eq!(stats.whisker_high, 5.0);
}

#[test]
fn gaps_are_ignored_and_nothing_finite_is_none() {
    let stats = BoxStats::of(&[2.0, f64::NAN, 4.0]).unwrap();
    assert_eq!(stats.median, 3.0);
    assert!(BoxStats::of(&[f64::NAN]).is_none());
}
