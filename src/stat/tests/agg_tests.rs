use super::Agg;

fn sample() -> Agg {
    Agg::by(["b", "a", "b", "a", "b"], &[10.0, 1.0, 30.0, 3.0, 20.0][..])
}

#[test]
fn groups_keep_first_appearance_order() {
    let (keys, counts) = sample().count();
    assert_eq!(keys, ["b", "a"]);
    assert_eq!(counts, [3.0, 2.0]);
}

#[test]
fn the_reducers_reduce() {
    assert_eq!(sample().sum().1, [60.0, 4.0]);
    assert_eq!(sample().mean().1, [20.0, 2.0]);
    assert_eq!(sample().min().1, [10.0, 1.0]);
    assert_eq!(sample().max().1, [30.0, 3.0]);
    assert_eq!(sample().median().1, [20.0, 2.0]);
}

#[test]
fn gaps_are_ignored_and_empty_groups_become_gaps() {
    let (keys, means) = Agg::by(["a", "a", "b"], &[1.0, f64::NAN, f64::NAN][..]).mean();
    assert_eq!(keys, ["a", "b"]);
    assert_eq!(means[0], 1.0);
    assert!(means[1].is_nan());
}

#[test]
#[should_panic(expected = "one key per value")]
fn mismatched_keys_and_values_panic() {
    Agg::by(["a"], &[1.0, 2.0][..]);
}
