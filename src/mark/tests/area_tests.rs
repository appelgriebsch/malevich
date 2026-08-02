use super::Area;
use crate::render::Color;

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<Area<'static>>();

#[test]
#[should_panic(expected = "equal length")]
fn mismatched_band_edges_panic() {
    Area::between(&[1.0][..], &[1.0, 2.0][..], &[1.0][..]);
}

#[test]
fn into_owned_detaches_from_borrowed_storage() {
    let values = vec![1.0, 2.0];
    let area = Area::y(values.as_slice()).color(Color::Green).into_owned();
    assert_ne!(area.high.as_slice().as_ptr(), values.as_ptr());
    assert_eq!(area.color, Some(Color::Green));
}

#[test]
fn debug_stays_curated() {
    let area = Area::y(&[1.0, 2.0, 3.0][..]);
    let debug = format!("{area:?}");
    assert!(debug.contains("points: 3"), "unexpected debug: {debug}");
    assert!(!debug.contains("1.0"), "debug dumps data: {debug}");
}
