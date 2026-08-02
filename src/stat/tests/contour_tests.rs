use super::contours;

/// Segments as ((x0, y0), (x1, y1)) pairs, decoded from the NaN-jointed layout.
fn segments(line: &super::Contour) -> Vec<((f64, f64), (f64, f64))> {
    assert!(line.x.len().is_multiple_of(3), "two endpoints plus a joint");
    line.x
        .chunks(3)
        .zip(line.y.chunks(3))
        .map(|(x, y)| {
            assert!(x[2].is_nan() && y[2].is_nan());
            ((x[0], y[0]), (x[1], y[1]))
        })
        .collect()
}

#[test]
fn a_vertical_gradient_yields_one_straight_line_at_the_interpolated_row() {
    // 3 columns * 4 rows, value = row index; the 1.5 contour sits halfway
    // between rows 1 and 2.
    let values: Vec<f64> = (0..12).map(|i| (i / 3) as f64).collect();
    let lines = contours(&values, 3, &[1.5]);
    let segs = segments(&lines[0]);
    assert_eq!(segs.len(), 2, "one segment per block column");
    for ((_, y0), (_, y1)) in segs {
        assert_eq!(y0, 1.5);
        assert_eq!(y1, 1.5);
    }
}

#[test]
fn a_peak_produces_a_closed_ring() {
    // A single interior peak: the contour around it must close — every endpoint
    // is shared by exactly two segments (interpolation is per-edge, so shared
    // endpoints match bit-for-bit).
    let mut values = vec![0.0; 25];
    values[2 * 5 + 2] = 4.0;
    let lines = contours(&values, 5, &[1.0]);
    let segs = segments(&lines[0]);
    assert!(!segs.is_empty());
    let mut counts = std::collections::HashMap::new();
    for (a, b) in segs {
        for point in [a, b] {
            *counts.entry(format!("{point:?}")).or_insert(0) += 1;
        }
    }
    assert!(counts.values().all(|&count| count == 2), "ring is closed");
}

#[test]
fn levels_outside_the_data_range_trace_nothing() {
    let values: Vec<f64> = (0..16).map(f64::from).collect();
    for line in contours(&values, 4, &[-1.0, 99.0]) {
        assert!(line.x.is_empty());
    }
}

#[test]
fn nan_values_leave_a_gap() {
    let values: Vec<f64> = (0..12).map(|i| (i / 3) as f64).collect();
    let full = segments(&contours(&values, 3, &[1.5])[0]).len();
    let mut holed = values;
    holed[2 * 3 + 1] = f64::NAN; // touches both blocks of the 1.5 crossing row
    let after = segments(&contours(&holed, 3, &[1.5])[0]).len();
    assert!(after < full);
}

#[test]
fn a_saddle_resolves_by_the_center_average() {
    // Opposite corners high: center mean 0.5 is below the 0.6 level, so the two
    // high corners stay separate — two segments hugging them.
    let values = [1.0, 0.0, 0.0, 1.0];
    let segs = segments(&contours(&values, 2, &[0.6])[0]);
    assert_eq!(segs.len(), 2);
    // Above the mean the connection flips but the count stays two.
    let segs = segments(&contours(&values, 2, &[0.4])[0]);
    assert_eq!(segs.len(), 2);
}
