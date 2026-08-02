use super::{Orientation, Rule};

#[test]
#[should_panic(expected = "finite position")]
fn non_finite_rules_panic() {
    Rule::h(f64::NAN);
}

#[test]
fn rules_carry_their_orientation() {
    assert_eq!(Rule::h(2.0).orientation, Orientation::Horizontal(2.0));
    assert_eq!(Rule::v(3.0).orientation, Orientation::Vertical(3.0));
}
