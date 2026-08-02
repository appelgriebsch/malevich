use ndarray::{Array1, Array2};

use crate::data::IntoSeries;

#[test]
fn a_contiguous_array_borrows_zero_copy() {
    let array = Array1::from_vec(vec![1.0, 2.0, 3.0]);
    let series = (&array).into_series();
    assert_eq!(
        series.as_slice().as_ptr(),
        array.as_slice().expect("contiguous").as_ptr()
    );
}

#[test]
fn a_strided_view_converts_with_the_right_values() {
    let matrix = Array2::from_shape_vec((3, 2), vec![1.0, 10.0, 2.0, 20.0, 3.0, 30.0]).unwrap();
    let column = matrix.column(1);
    let series = (&column).into_series();
    assert_eq!(series.as_slice(), &[10.0, 20.0, 30.0]);
}

#[test]
fn an_array_plots_end_to_end() {
    let array = Array1::linspace(0.0, std::f64::consts::TAU, 50).mapv(f64::sin);
    let chart = crate::line(&array).render(&crate::Frame::plain(40, 10));
    assert!(!chart.is_empty());
}
