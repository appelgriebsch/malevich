use super::Bars;
use crate::render::Color;

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<Bars<'static>>();

#[test]
#[should_panic(expected = "one category per value")]
fn mismatched_categories_and_values_panic() {
    Bars::new(["a", "b"], &[1.0][..]);
}

#[test]
fn explicit_colors_stick() {
    let bars = Bars::new(["a"], &[1.0][..]).color(Color::Blue);
    assert_eq!(bars.color, Some(Color::Blue));
}

#[test]
fn debug_stays_curated() {
    let bars = Bars::new(["a", "b", "c"], &[1.0, 2.0, 3.0][..]);
    let debug = format!("{bars:?}");
    assert!(debug.contains("bars: 3"), "unexpected debug: {debug}");
    assert!(!debug.contains("1.0"), "debug dumps data: {debug}");
}
