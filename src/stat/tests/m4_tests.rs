use super::{M4, m4};

fn wave(n: usize) -> (Vec<f64>, Vec<f64>) {
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..n)
        .map(|i| (i as f64 * 0.37).sin() * (1.0 + i as f64 * 0.001))
        .collect();
    (x, y)
}

#[test]
fn each_column_keeps_its_extremes_and_endpoints() {
    let mut aggregate = M4::new((0.0, 10.0), 1);
    for (x, y) in [(0.0, 5.0), (2.0, -3.0), (5.0, 9.0), (10.0, 1.0)] {
        aggregate.add(x, y);
    }
    let (x, y) = aggregate.emit();
    // first (0,5), min (2,-3), max (5,9), last (10,1) — in x order.
    assert_eq!(x, [0.0, 2.0, 5.0, 10.0]);
    assert_eq!(y, [5.0, -3.0, 9.0, 1.0]);
}

#[test]
fn merged_chunks_equal_one_sequential_pass() {
    let (x, y) = wave(10_000);
    let mut sequential = M4::new((0.0, 9_999.0), 160);
    for (&xv, &yv) in x.iter().zip(y.iter()) {
        sequential.add(xv, yv);
    }
    let mut merged = M4::new((0.0, 9_999.0), 160);
    for chunk in x.chunks(997).zip(y.chunks(997)).map(|(cx, cy)| {
        let mut partial = M4::new((0.0, 9_999.0), 160);
        for (&xv, &yv) in cx.iter().zip(cy.iter()) {
            partial.add(xv, yv);
        }
        partial
    }) {
        merged.merge(&chunk);
    }
    assert_eq!(sequential.emit(), merged.emit());
}

#[test]
fn gaps_survive_downsampling() {
    let x: Vec<f64> = (0..1000).map(|i| i as f64).collect();
    let mut y: Vec<f64> = (0..1000).map(|i| (i as f64 * 0.1).sin()).collect();
    y[500] = f64::NAN;
    let (_, emitted_y) = m4(&x, &y, 10).unwrap();
    assert!(
        emitted_y.iter().any(|value| value.is_nan()),
        "the gap vanished"
    );
}

#[test]
fn unsorted_x_refuses_to_downsample() {
    let x = [0.0, 5.0, 3.0, 8.0];
    let y = [1.0, 2.0, 3.0, 4.0];
    assert!(m4(&x, &y, 4).is_none());
}

#[test]
fn emitted_points_never_exceed_four_per_column() {
    let (x, y) = wave(50_000);
    let (ex, _) = m4(&x, &y, 100).unwrap();
    assert!(ex.len() <= 400, "emitted {} points", ex.len());
    assert!(ex.windows(2).all(|pair| pair[0] <= pair[1]), "not sorted");
}

#[test]
fn all_gap_series_emit_nothing() {
    assert!(m4(&[f64::NAN, f64::NAN], &[1.0, 2.0], 4).is_none());
}
