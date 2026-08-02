use super::Bins;

#[test]
fn values_land_in_their_bins_and_the_last_edge_is_inclusive() {
    let mut bins = Bins::new(0.0, 1.0, 4);
    for value in [0.0, 0.5, 1.0, 2.5, 3.9, 4.0] {
        bins.add(value);
    }
    assert_eq!(bins.counts(), [2, 1, 1, 2]);
}

#[test]
fn out_of_range_and_gap_values_are_ignored() {
    let mut bins = Bins::new(0.0, 1.0, 2);
    for value in [-0.1, 2.1, f64::NAN, 0.5] {
        bins.add(value);
    }
    assert_eq!(bins.counts(), [1, 0]);
}

#[test]
fn merged_chunks_equal_one_sequential_pass() {
    let values: Vec<f64> = (0..5_000).map(|i| ((i * 37) % 100) as f64 / 10.0).collect();
    let mut sequential = Bins::new(0.0, 1.0, 10);
    for &value in &values {
        sequential.add(value);
    }
    let mut merged = Bins::new(0.0, 1.0, 10);
    for chunk in values.chunks(613) {
        let mut partial = Bins::new(0.0, 1.0, 10);
        for &value in chunk {
            partial.add(value);
        }
        merged.merge(&partial);
    }
    assert_eq!(sequential, merged);
}

#[test]
fn auto_bins_cover_the_data_with_nice_edges() {
    let values: Vec<f64> = (0..1_000)
        .map(|i| ((i * 61) % 997) as f64 / 100.0)
        .collect();
    let bins = Bins::auto(&values, 60).unwrap();
    assert!(bins.start() <= 0.0);
    assert!(bins.end() >= 9.96);
    assert_eq!(bins.counts().iter().sum::<u64>(), 1_000);
    // Nice-decimal width: a short exact decimal.
    let width = format!("{}", bins.width());
    assert!(width.len() <= 5, "width {width} is not a nice decimal");
}

#[test]
fn constant_data_gets_one_bin() {
    let bins = Bins::auto(&[7.0; 42], 60).unwrap();
    assert_eq!(bins.counts(), [42]);
}

#[test]
fn no_finite_data_means_no_bins() {
    assert!(Bins::auto(&[f64::NAN], 60).is_none());
    assert!(Bins::auto(&[], 60).is_none());
}
