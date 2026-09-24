use super::{StepDirection, steps};

#[test]
fn every_direction_holds_values_flat_between_samples() {
    let x = [0.0, 1.0, 3.0];
    let y = [2.0, 5.0, 4.0];
    let (px, py) = steps(&x, &y, StepDirection::Post);
    assert_eq!(px, [0.0, 1.0, 1.0, 3.0, 3.0]);
    assert_eq!(py, [2.0, 2.0, 5.0, 5.0, 4.0]);
    let (qx, qy) = steps(&x, &y, StepDirection::Pre);
    assert_eq!(qx, [0.0, 0.0, 1.0, 1.0, 3.0]);
    assert_eq!(qy, [2.0, 5.0, 5.0, 4.0, 4.0]);
    let (mx, my) = steps(&x, &y, StepDirection::Mid);
    assert_eq!(mx, [0.0, 0.5, 0.5, 1.0, 2.0, 2.0, 3.0]);
    assert_eq!(my, [2.0, 2.0, 5.0, 5.0, 5.0, 4.0, 4.0]);
}

#[test]
fn gaps_break_the_steps_and_are_never_bridged() {
    let (x, y) = steps(&[0.0, 1.0, 2.0], &[1.0, f64::NAN, 3.0], StepDirection::Post);
    assert_eq!(x.len(), 3);
    assert!(y[1].is_nan());
    assert_eq!((x[2], y[2]), (2.0, 3.0));
    assert_eq!(steps(&[], &[], StepDirection::Mid), (vec![], vec![]));
}
