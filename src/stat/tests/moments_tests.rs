use super::Moments;

fn values() -> Vec<f64> {
    (0..2_000)
        .map(|i| (i as f64 * 0.91).sin() * 40.0 + i as f64 * 0.01)
        .collect()
}

#[test]
fn matches_the_naive_computation() {
    let values = values();
    let mut moments = Moments::new();
    for &value in &values {
        moments.add(value);
    }
    let naive_mean = values.iter().sum::<f64>() / values.len() as f64;
    let naive_variance =
        values.iter().map(|v| (v - naive_mean).powi(2)).sum::<f64>() / values.len() as f64;
    assert!((moments.mean().unwrap() - naive_mean).abs() < 1e-9);
    assert!((moments.variance().unwrap() - naive_variance).abs() < 1e-6);
    assert_eq!(moments.count(), 2_000);
}

#[test]
fn merged_chunks_equal_one_sequential_pass() {
    let values = values();
    let mut sequential = Moments::new();
    for &value in &values {
        sequential.add(value);
    }
    let mut merged = Moments::new();
    for chunk in values.chunks(313) {
        let mut partial = Moments::new();
        for &value in chunk {
            partial.add(value);
        }
        merged.merge(&partial);
    }
    assert!((sequential.mean().unwrap() - merged.mean().unwrap()).abs() < 1e-9);
    assert!((sequential.variance().unwrap() - merged.variance().unwrap()).abs() < 1e-6);
    assert_eq!(sequential.min(), merged.min());
    assert_eq!(sequential.max(), merged.max());
}

#[test]
fn gaps_do_not_count() {
    let mut moments = Moments::new();
    moments.add(1.0);
    moments.add(f64::NAN);
    moments.add(3.0);
    assert_eq!(moments.count(), 2);
    assert_eq!(moments.mean(), Some(2.0));
}

#[test]
fn empty_accumulators_answer_none() {
    let moments = Moments::new();
    assert_eq!(moments.mean(), None);
    assert_eq!(moments.min(), None);
}

#[test]
fn default_matches_new_so_extrema_start_unset() {
    let mut a = Moments::new();
    let mut b = Moments::default();
    for &v in &[5.0, -3.0, 8.0, -1.0] {
        a.add(v);
        b.add(v);
    }
    assert_eq!(a.min(), b.min());
    assert_eq!(a.max(), b.max());
    assert_eq!(b.min(), Some(-3.0));
    assert_eq!(b.max(), Some(8.0));
}
