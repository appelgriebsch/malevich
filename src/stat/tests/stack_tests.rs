use super::{StackOffset, StackOptions, StackOrder, stack, stack_with};

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

#[test]
fn negatives_stack_below_the_baseline_on_their_own_side() {
    let bands = stack(&[&[1.0, -2.0], &[3.0, -4.0]]);
    assert_eq!(bands[0], (vec![0.0, -2.0], vec![1.0, 0.0]));
    assert_eq!(bands[1], (vec![1.0, -6.0], vec![4.0, -2.0]));
    // Mixed signs at one position keep each sign on its side.
    let mixed = stack(&[&[1.0], &[-1.0], &[2.0]]);
    assert_eq!(mixed[0], (vec![0.0], vec![1.0]));
    assert_eq!(mixed[1], (vec![-1.0], vec![0.0]));
    assert_eq!(mixed[2], (vec![1.0], vec![3.0]));
    for (low, high) in &bands {
        assert!(low.iter().zip(high).all(|(l, h)| l <= h));
    }
}

#[test]
fn normalized_stacks_fill_the_unit_interval_and_leave_zero_totals_alone() {
    let options = StackOptions::new().offset(StackOffset::Normalize);
    let bands = stack_with(&[&[1.0, 0.0, -1.0], &[3.0, 0.0, -3.0]], options);
    assert_eq!(bands[0], (vec![0.0, 0.0, -0.25], vec![0.25, 0.0, 0.0]));
    assert_eq!(bands[1], (vec![0.25, 0.0, -1.0], vec![1.0, 0.0, -0.25]));
    // A gap normalizes like a zero: no thickness, no NaN.
    let gappy = stack_with(&[&[f64::NAN], &[2.0]], options);
    assert_eq!(gappy[0], (vec![0.0], vec![0.0]));
    assert_eq!(gappy[1], (vec![0.0], vec![1.0]));
}

#[test]
fn centered_stacks_straddle_zero() {
    let options = StackOptions::new().offset(StackOffset::Center);
    let bands = stack_with(&[&[2.0], &[2.0]], options);
    assert_eq!(bands[0], (vec![-2.0], vec![0.0]));
    assert_eq!(bands[1], (vec![0.0], vec![2.0]));
}

#[test]
fn sum_order_puts_the_largest_series_at_the_baseline_in_input_positions() {
    let options = StackOptions::new().order(StackOrder::Sum);
    let bands = stack_with(&[&[1.0, 1.0], &[10.0, 10.0]], options);
    assert_eq!(bands[1], (vec![0.0, 0.0], vec![10.0, 10.0]));
    assert_eq!(bands[0], (vec![10.0, 10.0], vec![11.0, 11.0]));
    // The defaults are exactly `stack`.
    let plain = [&[1.0, 2.0][..], &[3.0, 4.0][..]];
    assert_eq!(stack(&plain), stack_with(&plain, StackOptions::default()));
}
