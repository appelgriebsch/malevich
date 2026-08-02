use super::stack;

#[test]
fn each_series_sits_on_the_sum_of_the_previous() {
    let bands = stack(&[&[1.0, 2.0], &[10.0, 20.0], &[100.0, 200.0]]);
    assert_eq!(bands[0], (vec![0.0, 0.0], vec![1.0, 2.0]));
    assert_eq!(bands[1], (vec![1.0, 2.0], vec![11.0, 22.0]));
    assert_eq!(bands[2], (vec![11.0, 22.0], vec![111.0, 222.0]));
}

#[test]
fn gaps_and_short_series_contribute_nothing() {
    let bands = stack(&[&[1.0, f64::NAN, 3.0], &[10.0]]);
    assert_eq!(bands[0].1, [1.0, 0.0, 3.0]);
    assert_eq!(bands[1].1, [11.0, 0.0, 3.0]);
}

#[test]
fn no_series_stacks_to_nothing() {
    assert!(stack(&[]).is_empty());
}
