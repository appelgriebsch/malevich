//! Defensive geometry and allocation limits for caller-controlled render state.

use crate::{Error, Result};

/// A terminal dimension larger than `u16::MAX` is not meaningful to the supported
/// hosts and makes zero-area frames an otherwise easy way around an area limit.
pub(crate) const MAX_CELL_DIMENSION: usize = u16::MAX as usize;
/// Four million cells is already orders of magnitude larger than an interactive
/// terminal while keeping the infallible API's worst-case memory bounded.
pub(crate) const MAX_CELLS: usize = 4 * 1024 * 1024;
/// Device-pixel rendering has several transient copies; cap the raster separately.
pub(crate) const MAX_DEVICE_PIXELS: usize = 16 * 1024 * 1024;
/// No render call should construct an arbitrarily large escape/string payload.
pub(crate) const MAX_OUTPUT_BYTES: usize = 256 * 1024 * 1024;

pub(crate) fn dimension(what: &'static str, value: usize, limit: usize) -> Result<()> {
    if value > limit {
        return Err(Error::DimensionTooLarge {
            what,
            requested: value,
            limit,
        });
    }
    Ok(())
}

pub(crate) fn area(what: &'static str, width: usize, height: usize, limit: usize) -> Result<usize> {
    let requested = width.checked_mul(height).ok_or(Error::DimensionTooLarge {
        what,
        requested: usize::MAX,
        limit,
    })?;
    dimension(what, requested, limit)?;
    Ok(requested)
}

pub(crate) fn frame_cells(width: usize, height: usize) -> Result<usize> {
    dimension("frame width", width, MAX_CELL_DIMENSION)?;
    dimension("frame height", height, MAX_CELL_DIMENSION)?;
    area("frame cell count", width, height, MAX_CELLS)
}

pub(crate) fn reserve<T>(values: &mut Vec<T>, count: usize, what: &'static str) -> Result<()> {
    values
        .try_reserve_exact(count)
        .map_err(|_| Error::AllocationFailed { what })
}

pub(crate) fn reserve_string(value: &mut String, bytes: usize, what: &'static str) -> Result<()> {
    dimension(what, bytes, MAX_OUTPUT_BYTES)?;
    value
        .try_reserve_exact(bytes)
        .map_err(|_| Error::AllocationFailed { what })
}
