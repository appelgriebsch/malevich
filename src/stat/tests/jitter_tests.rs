use super::{jitter, van_der_corput};

#[test]
fn the_sequence_is_the_van_der_corput_one() {
    let first: Vec<f64> = (1..=8).map(van_der_corput).collect();
    assert_eq!(first, [0.5, 0.25, 0.75, 0.125, 0.625, 0.375, 0.875, 0.0625]);
}

#[test]
fn jitter_fills_the_band_evenly_keeps_gaps_and_is_deterministic() {
    let positions = vec![2.0; 64];
    let spread = jitter(&positions, 1.0);
    assert_eq!(spread, jitter(&positions, 1.0));
    assert!(spread.iter().all(|x| (1.5..2.5).contains(x)));
    // Every eighth of the band gets eight of the sixty-four points.
    let mut buckets = [0usize; 8];
    for x in &spread {
        buckets[(((x - 1.5) * 8.0) as usize).min(7)] += 1;
    }
    assert_eq!(buckets, [8; 8]);
    let gappy = jitter(&[1.0, f64::NAN], 0.5);
    assert!(gappy[1].is_nan());
    assert_eq!(jitter(&[3.0], 0.0), [3.0]);
}
