use super::{IntoSeries, Series};

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<Series<'static>>();

#[test]
fn borrowed_slices_are_zero_copy() {
    let values = vec![1.0, 2.0, 3.0];
    let series = values.as_slice().into_series();
    assert_eq!(series.as_slice().as_ptr(), values.as_ptr());
    let series = (&values).into_series();
    assert_eq!(series.as_slice().as_ptr(), values.as_ptr());
}

#[test]
fn owned_vectors_move_without_reallocating() {
    let values = vec![1.0, 2.0, 3.0];
    let pointer = values.as_ptr();
    let series = values.into_series();
    assert_eq!(series.as_slice().as_ptr(), pointer);
}

#[test]
fn other_scalar_types_convert_once() {
    assert_eq!([1i32, -2, 3].into_series().as_slice(), [1.0, -2.0, 3.0]);
    assert_eq!(vec![0.5f32, 1.5].into_series().as_slice(), [0.5, 1.5]);
    assert_eq!((&[7u8, 9][..]).into_series().as_slice(), [7.0, 9.0]);
}

#[test]
fn gaps_flow_through_ingestion_untouched() {
    let series = [1.0, f64::NAN, 3.0].into_series();
    assert_eq!(series.len(), 3);
    assert!(series.as_slice()[1].is_nan());
}

#[test]
fn the_extent_ignores_gaps_and_infinities() {
    let series = [f64::NAN, -2.0, f64::INFINITY, 5.0, f64::NEG_INFINITY].into_series();
    assert_eq!(series.extent(), Some((-2.0, 5.0)));
}

#[test]
fn the_extent_of_nothing_finite_is_none() {
    assert_eq!(Series::from_iter([] as [f64; 0]).extent(), None);
    assert_eq!([f64::NAN, f64::NAN].into_series().extent(), None);
}

#[test]
fn iterators_collect_into_owned_series() {
    let series: Series = (0..4).map(|i| i as f64 * 0.5).collect();
    assert_eq!(series.as_slice(), [0.0, 0.5, 1.0, 1.5]);
}

#[test]
fn into_owned_detaches_from_borrowed_storage() {
    let values = vec![1.0, 2.0];
    let owned = values.as_slice().into_series().into_owned();
    assert_ne!(owned.as_slice().as_ptr(), values.as_ptr());
    assert_eq!(owned.as_slice(), values.as_slice());
}

#[test]
fn a_series_passes_through_the_rim_unchanged() {
    let series = [1.0, 2.0].into_series();
    let same = series.clone().into_series();
    assert_eq!(same, series);
}
