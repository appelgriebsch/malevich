use super::{Bandwidth, KdeOptions, kde, kde_with};

#[test]
fn the_density_integrates_to_about_one() {
    let values: Vec<f64> = (0..2000)
        .map(|i| {
            let i = i as f64;
            ((i * 0.731).sin() + (i * 1.13).sin() + (i * 2.71).sin()) * 2.0
        })
        .collect();
    let (positions, densities) = kde(&values, 512).unwrap();
    let step = positions[1] - positions[0];
    let integral: f64 = densities.iter().sum::<f64>() * step;
    assert!((integral - 1.0).abs() < 0.02, "integral {integral}");
}

#[test]
fn the_mode_sits_near_the_data_center() {
    let values: Vec<f64> = (0..3000)
        .map(|i| {
            let i = i as f64;
            5.0 + ((i * 0.97).sin() + (i * 1.31).sin() + (i * 2.63).sin()) / 3.0
        })
        .collect();
    let (positions, densities) = kde(&values, 256).unwrap();
    let peak = densities
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(i, _)| positions[i])
        .unwrap();
    assert!((peak - 5.0).abs() < 0.5, "peak at {peak}");
}

#[test]
fn degenerate_and_empty_samples_behave() {
    assert!(kde(&[f64::NAN], 128).is_none());
    let (_, densities) = kde(&[7.0; 50], 128).unwrap();
    assert!(densities.iter().all(|d| d.is_finite()));
}

#[test]
fn a_degenerate_large_offset_sample_declines_without_panicking() {
    assert!(kde(&[1e20], 16).is_none());
    assert!(kde(&[1e20, 1e20, 1e20], 32).is_none());
}

#[test]
fn caller_selected_grid_is_bounded() {
    assert!(kde(&[1.0, 2.0], usize::MAX).is_none());
}

#[test]
fn gaps_do_not_change_the_finite_sample_density() {
    let finite = [1.0, 2.0, 3.0, 5.0, 8.0];
    let gappy = [
        1.0,
        f64::NAN,
        2.0,
        f64::INFINITY,
        3.0,
        f64::NEG_INFINITY,
        5.0,
        8.0,
    ];
    assert_eq!(kde(&gappy, 64), kde(&finite, 64));
}

#[test]
fn a_bounded_density_keeps_its_mass_inside_the_bound() {
    // Latencies pile up against zero: unbounded, the estimate leaks below it.
    let latencies: Vec<f64> = (0..400).map(|i| ((i % 37) as f64 * 0.11).powi(2)).collect();
    let (xs, unbounded) = kde(&latencies, 200).unwrap();
    let leaked: f64 = xs
        .iter()
        .zip(&unbounded)
        .filter(|(x, _)| **x < 0.0)
        .map(|(_, d)| d)
        .sum::<f64>()
        * (xs[1] - xs[0]);
    assert!(leaked > 0.01, "the unbounded estimate leaks {leaked}");

    let options = KdeOptions::new().bounds(Some(0.0), None);
    let (xs, bounded) = kde_with(&latencies, 200, options).unwrap().unwrap();
    assert_eq!(xs[0], 0.0, "the grid starts at the bound");
    let step = xs[1] - xs[0];
    let mass: f64 = bounded.iter().sum::<f64>() * step;
    assert!((mass - 1.0).abs() < 0.02, "bounded mass {mass}");
    // Reflection doubles the density at the bound relative to the unbounded
    // estimate there (the folded mass lands back on the edge).
    let (_, plain) = kde_with(&latencies, 200, KdeOptions::new().cut(0.0))
        .unwrap()
        .unwrap();
    assert!(
        bounded[0] > plain[0] * 1.5,
        "{} vs {}",
        bounded[0],
        plain[0]
    );
}

#[test]
fn bandwidth_rules_scale_the_smoothing_and_cumulative_rises_to_one() {
    let values: Vec<f64> = (0..300).map(|i| ((i * 7) % 300) as f64 / 30.0).collect();
    let (_, silverman) = kde(&values, 100).unwrap();
    let (_, half) = kde_with(
        &values,
        100,
        KdeOptions::new().bandwidth(Bandwidth::Scale(0.5)),
    )
    .unwrap()
    .unwrap();
    let (_, fixed) = kde_with(
        &values,
        100,
        KdeOptions::new().bandwidth(Bandwidth::Fixed(5.0)),
    )
    .unwrap()
    .unwrap();
    let peak = |d: &[f64]| d.iter().copied().fold(0.0, f64::max);
    assert!(
        peak(&half) > peak(&silverman),
        "less smoothing, sharper peak"
    );
    assert!(
        peak(&fixed) < peak(&silverman),
        "a wide fixed bandwidth flattens"
    );
    assert_eq!(
        kde(&values, 100).unwrap(),
        kde_with(&values, 100, KdeOptions::default())
            .unwrap()
            .unwrap(),
        "the defaults are exactly kde"
    );

    let (_, cumulative) = kde_with(&values, 400, KdeOptions::new().cumulative())
        .unwrap()
        .unwrap();
    assert_eq!(cumulative[0], 0.0);
    assert!(cumulative.windows(2).all(|pair| pair[1] >= pair[0]));
    assert!((cumulative[399] - 1.0).abs() < 0.02, "{}", cumulative[399]);
}

#[test]
fn invalid_kde_options_are_typed_errors() {
    for options in [
        KdeOptions::new().bandwidth(Bandwidth::Fixed(0.0)),
        KdeOptions::new().bandwidth(Bandwidth::Scale(f64::NAN)),
        KdeOptions::new().cut(-1.0),
        KdeOptions::new().bounds(Some(1.0), Some(0.0)),
        KdeOptions::new().bounds(Some(f64::NAN), None),
    ] {
        assert!(
            matches!(
                kde_with(&[1.0, 2.0], 10, options),
                Err(crate::Error::InvalidParameter { .. })
            ),
            "{options:?}"
        );
    }
    assert!(matches!(
        kde_with(
            &[1.0],
            crate::stat::MAX_STAT_ELEMENTS + 1,
            KdeOptions::new()
        ),
        Err(crate::Error::DimensionTooLarge { .. })
    ));
    // Values outside the bounds are not part of the sample.
    assert!(
        kde_with(&[-1.0, -2.0], 10, KdeOptions::new().bounds(Some(0.0), None))
            .unwrap()
            .is_none()
    );
}
