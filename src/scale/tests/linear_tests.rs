use super::Linear;

#[test]
fn map_and_unmap_are_inverses_in_both_range_directions() {
    for range in [(20.0, 80.0), (80.0, 20.0)] {
        let scale = Linear::new((-5.0, 15.0), range);
        for value in [-5.0, 0.0, 7.5, 15.0] {
            let round_trip = scale.unmap(scale.map(value));
            assert!((round_trip - value).abs() < 1e-12);
        }
    }
}

#[test]
fn degenerate_maps_are_stable_and_preserve_gaps() {
    let domain = Linear::new((4.0, 4.0), (0.0, 10.0));
    assert_eq!(domain.map(100.0), 5.0);
    assert!(domain.map(f64::NAN).is_nan());

    let range = Linear::new((2.0, 8.0), (5.0, 5.0));
    assert_eq!(range.unmap(100.0), 5.0);
    assert!(range.unmap(f64::NAN).is_nan());
}
