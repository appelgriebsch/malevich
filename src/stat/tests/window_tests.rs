use super::Window;

#[test]
fn trailing_means_smooth_without_a_warmup_gap() {
    let smoothed = Window::new(3).mean(&[3.0, 6.0, 9.0, 12.0]);
    assert_eq!(smoothed, [3.0, 4.5, 6.0, 9.0]);
}

#[test]
fn gaps_are_excluded_and_all_gap_windows_stay_gaps() {
    let smoothed = Window::new(2).mean(&[1.0, f64::NAN, 5.0]);
    assert_eq!(smoothed[0], 1.0);
    assert_eq!(smoothed[1], 1.0);
    assert_eq!(smoothed[2], 5.0);
    let gaps = Window::new(1).mean(&[f64::NAN]);
    assert!(gaps[0].is_nan());
}

#[test]
fn the_other_reducers_reduce() {
    let window = Window::new(2);
    assert_eq!(window.sum(&[1.0, 2.0, 3.0]), [1.0, 3.0, 5.0]);
    assert_eq!(window.min(&[3.0, 1.0, 2.0]), [3.0, 1.0, 1.0]);
    assert_eq!(window.max(&[1.0, 3.0, 2.0]), [1.0, 3.0, 3.0]);
}
