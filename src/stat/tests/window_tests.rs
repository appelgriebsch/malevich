use super::{Window, WindowAnchor};
use crate::stat::Reducer;

#[test]
fn trailing_means_smooth_without_a_warmup_gap() {
    let smoothed = Window::new(3).mean(&[3.0, 6.0, 9.0, 12.0]);
    assert_eq!(smoothed, [3.0, 4.5, 6.0, 9.0]);
}

#[test]
fn gaps_are_excluded_and_all_gap_windows_stay_gaps() {
    let smoothed = Window::new(2).mean(&[1.0, f64::NAN, 5.0]);
    assert_eq!(smoothed[0], 1.0);
    assert_eq!(smoothed[1], 1.0);
    assert_eq!(smoothed[2], 5.0);
    let gaps = Window::new(1).mean(&[f64::NAN]);
    assert!(gaps[0].is_nan());
}

#[test]
fn the_other_reducers_reduce() {
    let window = Window::new(2);
    assert_eq!(window.sum(&[1.0, 2.0, 3.0]), [1.0, 3.0, 5.0]);
    assert_eq!(window.min(&[3.0, 1.0, 2.0]), [3.0, 1.0, 1.0]);
    assert_eq!(window.max(&[1.0, 3.0, 2.0]), [1.0, 3.0, 3.0]);
}

#[test]
fn optimized_rolling_strategies_match_one_shot_reduction() {
    let values: Vec<f64> = (0..257)
        .map(|index| match index % 29 {
            0 => f64::NAN,
            1 => f64::INFINITY,
            _ => ((index * 37) % 101) as f64 - 50.0,
        })
        .collect();
    let reducers = [
        Reducer::Count,
        Reducer::Sum,
        Reducer::Mean,
        Reducer::Min,
        Reducer::Max,
        Reducer::Median,
        Reducer::Percentile(0.9),
        Reducer::Deviation,
        Reducer::Variance,
        Reducer::StdErr,
        Reducer::First,
        Reducer::Last,
    ];

    for size in [1, 2, 7, 64, 999] {
        let window = Window::new(size);
        for reducer in reducers {
            let actual = window.reduce(&values, reducer);
            let expected: Vec<f64> = (0..values.len())
                .map(|end| {
                    let start = (end + 1).saturating_sub(size);
                    reducer.reduce(&values[start..=end])
                })
                .collect();
            for (index, (&actual, &expected)) in actual.iter().zip(&expected).enumerate() {
                assert!(
                    (actual.is_nan() && expected.is_nan())
                        || (actual - expected).abs()
                            <= f64::EPSILON * 32.0 * actual.abs().max(expected.abs()).max(1.0),
                    "size {size}, reducer {reducer:?}, index {index}: {actual:?} != {expected:?}"
                );
            }
        }
    }
}

#[test]
fn an_overflowing_mean_does_not_poison_later_windows() {
    let means = Window::new(2).mean(&[f64::MAX, f64::MAX, -f64::MAX, 1.0]);
    assert_eq!(means[0], f64::MAX);
    assert_eq!(means[1], f64::MAX);
    assert_eq!(means[2], 0.0);
    assert_eq!(means[3], -f64::MAX / 2.0);
}

#[test]
#[should_panic(expected = "Reducer::Percentile requires a position in [0, 1]")]
fn invalid_percentiles_are_rejected_even_for_empty_input() {
    Window::new(3).reduce(&[], Reducer::Percentile(2.0));
}

#[test]
fn anchors_shift_the_window_and_strict_windows_gap_the_ends() {
    let values = [1.0, 2.0, 3.0, 4.0, 5.0];
    let trailing = Window::new(3).mean(&values);
    assert_eq!(trailing, [1.0, 1.5, 2.0, 3.0, 4.0]);
    let centered = Window::new(3).anchor(WindowAnchor::Middle).mean(&values);
    assert_eq!(centered, [1.5, 2.0, 3.0, 4.0, 4.5]);
    let leading = Window::new(3).anchor(WindowAnchor::Start).mean(&values);
    assert_eq!(leading, [2.0, 3.0, 4.0, 4.5, 5.0]);
    // An even size leans left when centered: [i-1, i+2].
    let even = Window::new(4).anchor(WindowAnchor::Middle).sum(&values);
    assert_eq!(even, [6.0, 10.0, 14.0, 12.0, 9.0]);

    let strict = Window::new(3).strict().mean(&values);
    assert!(strict[0].is_nan() && strict[1].is_nan());
    assert_eq!(&strict[2..], [2.0, 3.0, 4.0]);
    let strict_centered = Window::new(3)
        .anchor(WindowAnchor::Middle)
        .strict()
        .mean(&values);
    assert!(strict_centered[0].is_nan() && strict_centered[4].is_nan());
    assert_eq!(&strict_centered[1..4], [2.0, 3.0, 4.0]);
    // A window wider than the series reduces the whole series everywhere.
    let wide = Window::new(9).anchor(WindowAnchor::Middle).max(&values);
    assert_eq!(wide, [5.0; 5]);
    assert!(
        Window::new(9)
            .strict()
            .max(&values)
            .iter()
            .all(|v| v.is_nan())
    );
}

#[test]
fn anchored_windows_match_one_shot_reduction_of_their_exact_span() {
    let values: Vec<f64> = (0..97)
        .map(|index| match index % 13 {
            0 => f64::NAN,
            _ => ((index * 31) % 47) as f64 - 20.0,
        })
        .collect();
    for size in [1usize, 2, 3, 8, 50] {
        for anchor in [WindowAnchor::Start, WindowAnchor::Middle, WindowAnchor::End] {
            let back = match anchor {
                WindowAnchor::Start => 0,
                WindowAnchor::Middle => (size - 1) / 2,
                WindowAnchor::End => size - 1,
            };
            for reducer in [
                Reducer::Mean,
                Reducer::Median,
                Reducer::Deviation,
                Reducer::Last,
            ] {
                let actual = Window::new(size).anchor(anchor).reduce(&values, reducer);
                for (position, &got) in actual.iter().enumerate() {
                    let start = position.saturating_sub(back);
                    let end = (position + size - back).min(values.len());
                    let expected = reducer.reduce(&values[start..end]);
                    assert!(
                        (got.is_nan() && expected.is_nan()) || (got - expected).abs() < 1e-9,
                        "size {size}, {anchor:?}, {reducer:?}, at {position}: {got} vs {expected}"
                    );
                }
            }
        }
    }
}
