//! `ndarray` ingestion: one-dimensional arrays and views enter as series.

use ndarray::{ArrayBase, Data, Ix1};

use super::{IntoSeries, Series};

/// Any one-dimensional `ndarray` array or view of `f64`.
///
/// Contiguous storage borrows zero-copy; strided views (a column of a matrix, a
/// stepped slice) convert and copy once, like every other non-slice input.
impl<'a, S> IntoSeries<'a> for &'a ArrayBase<S, Ix1>
where
    S: Data<Elem = f64>,
{
    fn into_series(self) -> Series<'a> {
        match self.as_slice() {
            Some(slice) => slice.into_series(),
            None => self.iter().copied().collect(),
        }
    }
}

#[cfg(test)]
#[path = "tests/ndarray_tests.rs"]
mod tests;
