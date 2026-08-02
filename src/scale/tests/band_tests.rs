use super::Band;

#[test]
fn bands_partition_the_range_evenly() {
    let band = Band::new(4, (0.0, 100.0));
    assert_eq!(band.count(), 4);
    let step = band.step();
    for index in 0..3 {
        let gap = band.position(index + 1) - band.position(index);
        assert!((gap - step).abs() < 1e-9);
    }
    assert!(band.bandwidth() < step);
    assert!(band.position(0) > 0.0);
    assert!(band.position(3) + band.bandwidth() < 100.0);
}

#[test]
fn centers_sit_inside_their_bands() {
    let band = Band::new(3, (0.0, 30.0));
    for index in 0..3 {
        let center = band.center(index);
        assert!(center > band.position(index));
        assert!(center < band.position(index) + band.bandwidth());
    }
}

#[test]
fn a_single_band_fills_most_of_the_range() {
    let band = Band::new(1, (0.0, 10.0));
    assert!(band.bandwidth() > 5.0);
    assert!(band.bandwidth() < 10.0);
}

#[test]
fn zero_bands_do_not_divide_by_zero() {
    let band = Band::new(0, (0.0, 10.0));
    assert_eq!(band.bandwidth(), 0.0);
    assert_eq!(band.count(), 0);
}
