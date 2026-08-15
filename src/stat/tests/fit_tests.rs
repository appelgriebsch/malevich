use super::Fit;

#[test]
fn a_perfect_line_recovers_its_coefficients_exactly() {
    let x = [1.0, 2.0, 3.0, 4.0];
    let y: Vec<f64> = x.iter().map(|v| 3.0 * v - 5.0).collect();
    let fit = Fit::xy(&x, &y);
    assert_eq!(fit.slope(), Some(3.0));
    assert_eq!(fit.intercept(), Some(-5.0));
    assert_eq!(fit.r_squared(), Some(1.0));
    assert_eq!(fit.predict(10.0), Some(25.0));
}

#[test]
fn a_hand_checked_fit_matches_the_textbook_formulas() {
    // x = [0,1,2,3], y = [1,3,2,5]: x̄ = 1.5, ȳ = 2.75, Sxy = 5.5, Sxx = 5,
    // Syy = 8.75 → slope 1.1, intercept 1.1, R² = 5.5²/(5 × 8.75) = 121/175.
    let fit = Fit::xy(&[0.0, 1.0, 2.0, 3.0], &[1.0, 3.0, 2.0, 5.0]);
    assert!((fit.slope().unwrap() - 1.1).abs() < 1e-12);
    assert!((fit.intercept().unwrap() - 1.1).abs() < 1e-12);
    assert!((fit.r_squared().unwrap() - 121.0 / 175.0).abs() < 1e-12);
}

#[test]
fn merging_chunks_equals_one_pass() {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.37).collect();
    let y: Vec<f64> = x
        .iter()
        .enumerate()
        .map(|(i, v)| 2.0 * v + 1.0 + ((i * 37) % 17) as f64 * 0.1)
        .collect();

    let whole = Fit::xy(&x, &y);
    let mut merged = Fit::xy(&x[..33], &y[..33]);
    merged.merge(&Fit::xy(&x[33..70], &y[33..70]));
    merged.merge(&Fit::xy(&x[70..], &y[70..]));

    assert_eq!(merged.count(), whole.count());
    assert!((merged.slope().unwrap() - whole.slope().unwrap()).abs() < 1e-12);
    assert!((merged.intercept().unwrap() - whole.intercept().unwrap()).abs() < 1e-12);
    assert!((merged.r_squared().unwrap() - whole.r_squared().unwrap()).abs() < 1e-12);
}

#[test]
fn merging_with_the_empty_accumulator_is_the_identity() {
    let fit = Fit::xy(&[1.0, 2.0, 3.0], &[2.0, 4.0, 5.0]);
    let mut left = Fit::new();
    left.merge(&fit);
    let mut right = fit;
    right.merge(&Fit::new());
    assert_eq!(left.slope(), fit.slope());
    assert_eq!(right.slope(), fit.slope());
}

#[test]
fn offset_data_does_not_cancel_catastrophically() {
    // Unix-timestamp-sized x with a tiny slope: the naive Σx² formula loses
    // this; Welford accumulation keeps it.
    let base = 1.7e9;
    let x: Vec<f64> = (0..1000).map(|i| base + i as f64).collect();
    let y: Vec<f64> = x.iter().map(|v| 0.001 * (v - base) + 42.0).collect();
    let fit = Fit::xy(&x, &y);
    assert!((fit.slope().unwrap() - 0.001).abs() < 1e-9);
    assert!((fit.r_squared().unwrap() - 1.0).abs() < 1e-9);
}

#[test]
fn gaps_and_degenerate_shapes_answer_none() {
    let mut fit = Fit::new();
    assert_eq!(fit.slope(), None);
    fit.add(1.0, 2.0);
    assert_eq!(fit.slope(), None, "one point is not a line");
    fit.add(1.0, 5.0);
    assert_eq!(fit.slope(), None, "no x spread is not a line");

    let gappy = Fit::xy(&[1.0, f64::NAN, 2.0], &[1.0, 9.9, f64::NAN]);
    assert_eq!(gappy.count(), 1, "non-finite pairs are gaps");
}

#[test]
fn the_standard_error_narrows_at_the_mean_and_with_more_data() {
    let x: Vec<f64> = (0..50).map(f64::from).collect();
    let y: Vec<f64> = x
        .iter()
        .map(|v| 0.5 * v + ((v * 7.0) % 3.0) - 1.0)
        .collect();
    let fit = Fit::xy(&x, &y);
    let center = fit.standard_error(24.5).unwrap();
    let edge = fit.standard_error(49.0).unwrap();
    assert!(center < edge, "the band must flare away from the mean");

    let mut more = fit;
    more.merge(&Fit::xy(&x, &y));
    assert!(
        more.standard_error(24.5).unwrap() < center,
        "more data must narrow the band"
    );
}

#[test]
fn constant_y_is_a_perfect_horizontal_fit() {
    let fit = Fit::xy(&[1.0, 2.0, 3.0], &[4.0, 4.0, 4.0]);
    assert_eq!(fit.slope(), Some(0.0));
    assert_eq!(fit.intercept(), Some(4.0));
    assert_eq!(fit.r_squared(), Some(1.0));
}
