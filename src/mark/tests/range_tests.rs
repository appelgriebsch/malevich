use super::Range;

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<Range<'static>>();

#[test]
#[should_panic(expected = "equal length")]
fn mismatched_interval_edges_panic() {
    Range::xy(&[1.0][..], &[1.0, 2.0][..], &[1.0][..]);
}

#[test]
#[should_panic(expected = "matching the range length")]
fn mismatched_body_channels_panic() {
    let _ = Range::y(&[1.0][..], &[2.0][..]).body(&[1.0, 2.0][..], &[1.5, 2.5][..]);
}

#[test]
fn into_owned_detaches_from_borrowed_storage() {
    let low = vec![1.0];
    let high = vec![2.0];
    let range = Range::y(low.as_slice(), high.as_slice())
        .marker(&[1.5][..])
        .into_owned();
    assert_ne!(range.low.as_slice().as_ptr(), low.as_ptr());
    assert!(range.marker.is_some());
}

#[test]
fn debug_stays_curated() {
    let range = Range::y(&[1.0, 2.0][..], &[3.0, 4.0][..]);
    let debug = format!("{range:?}");
    assert!(debug.contains("intervals: 2"), "unexpected debug: {debug}");
    assert!(!debug.contains("1.0"), "debug dumps data: {debug}");
}
