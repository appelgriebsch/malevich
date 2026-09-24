use super::{BoxStats, Whiskers};

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

#[test]
fn whisker_rules_move_the_whiskers_and_keep_the_quartiles() {
    let values: Vec<f64> = (1..=20).map(f64::from).chain([60.0]).collect();
    let tukey = BoxStats::of(&values).unwrap();
    assert_eq!(tukey.outliers, [60.0]);
    assert_eq!(
        tukey,
        BoxStats::of_with(&values, Whiskers::Tukey(1.5)).unwrap()
    );

    let minmax = BoxStats::of_with(&values, Whiskers::MinMax).unwrap();
    assert_eq!((minmax.whisker_low, minmax.whisker_high), (1.0, 60.0));
    assert!(minmax.outliers.is_empty());
    assert_eq!(
        (minmax.q1, minmax.median, minmax.q3),
        (tukey.q1, tukey.median, tukey.q3)
    );

    let inner = BoxStats::of_with(&values, Whiskers::Percentiles(0.1, 0.9)).unwrap();
    assert_eq!(inner.whisker_low, 3.0);
    assert_eq!(inner.whisker_high, 19.0);
    assert_eq!(inner.outliers, [1.0, 2.0, 20.0, 60.0]);

    let wide = BoxStats::of_with(&values, Whiskers::Tukey(5.0)).unwrap();
    assert!(wide.outliers.is_empty());
}

#[test]
#[should_panic(expected = "valid whisker rule")]
fn descending_whisker_percentiles_are_misuse() {
    let _ = BoxStats::of_with(&[1.0, 2.0], Whiskers::Percentiles(0.9, 0.1));
}
