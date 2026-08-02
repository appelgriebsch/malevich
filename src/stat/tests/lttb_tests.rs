use super::lttb;

#[test]
fn keeps_both_endpoints_and_hits_the_target() {
    let x: Vec<f64> = (0..10_000).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|v| (v * 0.05).sin()).collect();
    let (dx, dy) = lttb(&x, &y, 200);
    assert_eq!(dx.len(), 200);
    assert_eq!(dy.len(), 200);
    assert_eq!(dx[0], 0.0);
    assert_eq!(dx[199], 9_999.0);
    assert!(dx.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn small_series_pass_through() {
    let x = [1.0, 2.0, 3.0];
    let y = [4.0, 5.0, 6.0];
    let (dx, dy) = lttb(&x, &y, 100);
    assert_eq!(dx, x);
    assert_eq!(dy, y);
}

#[test]
fn spikes_survive_shape_preservation() {
    let x: Vec<f64> = (0..5_000).map(|i| i as f64).collect();
    let mut y = vec![0.0; 5_000];
    y[2_500] = 100.0;
    let (_, dy) = lttb(&x, &y, 50);
    assert!(dy.contains(&100.0), "spike lost");
}

#[test]
fn gaps_are_filtered_out() {
    let x = [0.0, 1.0, 2.0, 3.0];
    let y = [1.0, f64::NAN, 3.0, 4.0];
    let (dx, _) = lttb(&x, &y, 100);
    assert_eq!(dx, [0.0, 2.0, 3.0]);
}
