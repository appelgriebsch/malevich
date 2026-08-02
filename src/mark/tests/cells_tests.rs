use super::Cells;

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<Cells<'static>>();

#[test]
#[should_panic(expected = "divide the value count")]
fn ragged_grids_panic() {
    Cells::matrix(3, &[1.0, 2.0][..]);
}

#[test]
fn debug_stays_curated() {
    let cells = Cells::matrix(2, &[1.0, 2.0, 3.0, 4.0][..]);
    let debug = format!("{cells:?}");
    assert!(debug.contains("columns: 2") && debug.contains("rows: 2"));
    assert!(!debug.contains("1.0"), "debug dumps data: {debug}");
}
