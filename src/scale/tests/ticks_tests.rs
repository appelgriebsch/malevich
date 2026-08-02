use super::Ticks;

fn labels(ticks: &Ticks) -> Vec<&str> {
    ticks.iter().map(|tick| tick.label.as_str()).collect()
}

#[test]
fn spans_zero_to_one_hundred_with_multiples_of_twenty() {
    let ticks = Ticks::linear(0.0, 100.0, 6);
    assert_eq!(labels(&ticks), ["0", "20", "40", "60", "80", "100"]);
}

#[test]
fn spans_the_unit_interval_with_quarters() {
    let ticks = Ticks::linear(0.0, 1.0, 5);
    assert_eq!(labels(&ticks), ["0.00", "0.25", "0.50", "0.75", "1.00"]);
}

#[test]
fn spans_a_symmetric_range_with_uniform_decimals() {
    let ticks = Ticks::linear(-1.0, 1.0, 5);
    assert_eq!(labels(&ticks), ["-1.0", "-0.5", "0.0", "0.5", "1.0"]);
}

#[test]
fn reversed_bounds_behave_like_sorted_bounds() {
    assert_eq!(Ticks::linear(100.0, 0.0, 6), Ticks::linear(0.0, 100.0, 6));
}

#[test]
fn equal_bounds_yield_a_single_tick() {
    let ticks = Ticks::linear(5.0, 5.0, 7);
    assert_eq!(labels(&ticks), ["5"]);
    assert_eq!(ticks.as_slice()[0].value, 5.0);
    assert_eq!(ticks.step(), 0.0);
}

#[test]
#[should_panic(expected = "finite bounds")]
fn rejects_non_finite_bounds() {
    Ticks::linear(f64::NAN, 1.0, 5);
}

#[test]
fn a_target_below_two_is_treated_as_two() {
    let ticks = Ticks::linear(0.0, 10.0, 0);
    assert!(ticks.len() >= 2);
}

/// A deterministic grid of ranges exercising magnitudes from 1e-6 to 1e5.
fn sweep() -> Vec<(f64, f64, usize)> {
    let mut cases = Vec::new();
    for &lo in &[-3.7, 0.0, 0.123, 55.0, -1000.0] {
        for &span in &[1e-6, 0.9, 3.0, 47.0, 1e5] {
            for &target in &[2usize, 3, 5, 8, 13] {
                cases.push((lo, lo + span, target));
            }
        }
    }
    cases
}

#[test]
fn ticks_are_ascending_and_uniformly_spaced() {
    for (lo, hi, target) in sweep() {
        let ticks = Ticks::linear(lo, hi, target);
        let values: Vec<f64> = ticks.iter().map(|tick| tick.value).collect();
        for pair in values.windows(2) {
            let diff = pair[1] - pair[0];
            assert!(diff > 0.0, "not ascending in [{lo}, {hi}]");
            // Values are correctly rounded individually, so their differences can
            // wobble by a few ulps of the value magnitude (visible when a tiny range
            // sits at a large offset). The decimal spacing itself is exact.
            let tolerance = 8.0 * f64::EPSILON * pair[0].abs().max(pair[1].abs()).max(ticks.step());
            assert!(
                (diff - ticks.step()).abs() <= tolerance,
                "non-uniform spacing in [{lo}, {hi}]"
            );
        }
    }
}

/// Splits a label into its numeric part and SI prefix factor.
fn decode(label: &str) -> (&str, f64) {
    let prefixes = [
        ('k', 1e3),
        ('M', 1e6),
        ('G', 1e9),
        ('T', 1e12),
        ('\u{00B5}', 1e-6),
        ('n', 1e-9),
        ('p', 1e-12),
    ];
    for (suffix, factor) in prefixes {
        if let Some(numeric) = label.strip_suffix(suffix) {
            return (numeric, factor);
        }
    }
    (label, 1.0)
}

#[test]
fn labels_parse_back_to_their_exact_values() {
    for (lo, hi, target) in sweep() {
        let ticks = Ticks::linear(lo, hi, target);
        for tick in &ticks {
            let (numeric, factor) = decode(&tick.label);
            let parsed: f64 = numeric.parse().unwrap();
            assert_eq!(
                parsed * factor,
                tick.value,
                "label {:?} does not decode to value {} in [{lo}, {hi}]",
                tick.label,
                tick.value
            );
        }
    }
}

#[test]
fn large_axes_share_one_si_prefix() {
    let ticks = Ticks::linear(0.0, 10_000_000.0, 5);
    let labels: Vec<&str> = ticks.iter().map(|tick| tick.label.as_str()).collect();
    assert_eq!(labels, ["0", "2.5M", "5.0M", "7.5M", "10.0M"]);
}

#[test]
fn tiny_axes_use_micro_prefixes() {
    let ticks = Ticks::linear(0.0, 0.0004, 4);
    let labels: Vec<&str> = ticks.iter().map(|tick| tick.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "0",
            "100\u{00B5}",
            "200\u{00B5}",
            "300\u{00B5}",
            "400\u{00B5}"
        ]
    );
}

#[test]
fn labels_share_one_fraction_width_and_never_render_negative_zero() {
    for (lo, hi, target) in sweep() {
        let ticks = Ticks::linear(lo, hi, target);
        // Zero keeps its bare "0" on prefixed axes — the deliberate exception.
        let widths: Vec<usize> = ticks
            .iter()
            .filter(|tick| tick.value != 0.0)
            .map(|tick| decode(&tick.label).0.split('.').nth(1).map_or(0, str::len))
            .collect();
        assert!(
            widths.windows(2).all(|pair| pair[0] == pair[1]),
            "mixed fraction widths in [{lo}, {hi}]: {:?}",
            labels(&ticks)
        );
        for tick in &ticks {
            assert!(!tick.label.starts_with("-0.0") || tick.value != 0.0);
            assert_ne!(tick.label, "-0");
        }
    }
}

#[test]
fn ticks_stay_within_one_step_of_the_data_range() {
    for (lo, hi, target) in sweep() {
        let ticks = Ticks::linear(lo, hi, target);
        let first = ticks.as_slice().first().unwrap().value;
        let last = ticks.as_slice().last().unwrap().value;
        let step = ticks.step();
        // The near side is guaranteed by construction; overshoot is a scored
        // trade-off, so it gets a looser bound.
        assert!(
            first <= lo + step,
            "first tick starts past the data in [{lo}, {hi}]"
        );
        assert!(
            last >= hi - step,
            "last tick ends before the data in [{lo}, {hi}]"
        );
        assert!(first >= lo - 2.0 * step, "first tick far below {lo}");
        assert!(last <= hi + 2.0 * step, "last tick far above {hi}");
    }
}

#[test]
fn the_step_is_a_preferred_mantissa_times_a_small_skip() {
    // The algorithm's contract: step = skip * q * 10^z with q from the preferred set
    // and a small integer skip (skip > 1 is a heavily penalized last resort).
    for (lo, hi, target) in sweep() {
        let ticks = Ticks::linear(lo, hi, target);
        let step = ticks.step();
        let magnitude = 10f64.powi(step.log10().floor() as i32);
        let mantissa = step / magnitude;
        let preferred = [1.0, 2.0, 2.5, 3.0, 4.0, 5.0, 10.0];
        let explained = preferred.iter().any(|q| {
            let ratio = mantissa / q;
            let skip = ratio.round();
            (1.0..=20.0).contains(&skip) && (ratio - skip).abs() < 1e-6
        });
        assert!(
            explained,
            "step {step} has mantissa {mantissa} in [{lo}, {hi}]"
        );
    }
}

#[test]
fn the_count_stays_near_the_target() {
    for (lo, hi, target) in sweep() {
        let ticks = Ticks::linear(lo, hi, target);
        assert!(ticks.len() >= 2);
        assert!(
            ticks.len() <= 3 * target,
            "asked for ~{target}, got {} in [{lo}, {hi}]",
            ticks.len()
        );
    }
}

#[test]
fn extreme_but_finite_bounds_do_not_panic() {
    let _ = Ticks::linear(-f64::MAX, f64::MAX, 6);
    let _ = Ticks::linear(f64::MIN_POSITIVE, f64::MAX, 8);
    let _ = Ticks::linear(-1e300, 1e300, 100);
}
