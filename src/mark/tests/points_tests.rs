use super::{PointStyle, Points};
use crate::render::Color;

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<Points<'static>>();
const _: () = assert_send_sync::<PointStyle>();

#[test]
#[should_panic(expected = "equal length")]
fn paired_series_of_unequal_lengths_panic() {
    Points::xy(&[1.0, 2.0][..], &[1.0][..]);
}

#[test]
fn explicit_colors_stick() {
    let points = Points::y(&[1.0][..]).color(Color::Green);
    assert_eq!(points.color, Some(Color::Green));
}

#[test]
fn marker_styles_are_explicit_and_dots_are_the_default() {
    assert_eq!(Points::y([1.0]).style, PointStyle::Dot);
    assert_eq!(
        Points::y([1.0]).style(PointStyle::Cross).style,
        PointStyle::Cross
    );
}

#[test]
fn into_owned_detaches_from_borrowed_storage() {
    let values = vec![1.0, 2.0];
    let points = Points::y(values.as_slice()).into_owned();
    assert_ne!(points.y.as_slice().as_ptr(), values.as_ptr());
    assert_eq!(points.y.as_slice(), values.as_slice());
}

#[test]
fn debug_stays_curated() {
    let points = Points::y(&[1.0, 2.0, 3.0][..]);
    let debug = format!("{points:?}");
    assert!(debug.contains("points: 3"), "unexpected debug: {debug}");
    assert!(!debug.contains("1.0"), "debug dumps data: {debug}");
}
