use super::dodge;

#[test]
fn series_center_on_their_band_and_touch() {
    let positions = dodge(&[&[0.0; 3], &[0.0; 3], &[0.0; 3]], 0.3);
    assert_eq!(positions[0], [-0.3, 0.7, 1.7]);
    assert_eq!(positions[1], [0.0, 1.0, 2.0]);
    assert_eq!(positions[2], [0.3, 1.3, 2.3]);
}

#[test]
fn two_series_straddle_the_band_center() {
    let positions = dodge(&[&[0.0; 2], &[0.0; 2]], 0.32);
    assert_eq!(positions[0], [-0.16, 0.84]);
    assert_eq!(positions[1], [0.16, 1.16]);
}

#[test]
fn one_series_sits_on_the_band_centers() {
    assert_eq!(dodge(&[&[5.0, 6.0]], 0.9), [[0.0, 1.0]]);
}

#[test]
fn short_series_get_their_own_length_and_none_dodges_to_nothing() {
    let positions = dodge(&[&[0.0; 3], &[0.0]], 1.0);
    assert_eq!(positions[1], [0.5]);
    assert!(dodge(&[], 1.0).is_empty());
}
