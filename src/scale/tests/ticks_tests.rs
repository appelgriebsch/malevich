use super::{TickOptions, Ticks};
use crate::scale::Unit;

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
    assert_eq!(ticks.step(), None);
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
        let step = ticks.step().expect("linear ticks have a uniform step");
        let values: Vec<f64> = ticks.iter().map(|tick| tick.value).collect();
        for pair in values.windows(2) {
            let diff = pair[1] - pair[0];
            assert!(diff > 0.0, "not ascending in [{lo}, {hi}]");
            // Values are correctly rounded individually, so their differences can
            // wobble by a few ulps of the value magnitude (visible when a tiny range
            // sits at a large offset). The decimal spacing itself is exact.
            let tolerance = 8.0 * f64::EPSILON * pair[0].abs().max(pair[1].abs()).max(step);
            assert!(
                (diff - step).abs() <= tolerance,
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

/// The value a label names, parsed in one step — `527.69847019259p` as
/// `527.69847019259e-12` — so the check never rounds through a product.
fn decoded(label: &str) -> f64 {
    let (numeric, factor) = decode(label);
    if factor == 1.0 {
        return numeric.parse().unwrap_or_else(|_| panic!("{label} parses"));
    }
    let shift = factor.log10().round() as i32;
    format!("{numeric}e{shift}")
        .parse()
        .unwrap_or_else(|_| panic!("{label} parses"))
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
        let step = ticks.step().expect("linear ticks have a uniform step");
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
        let step = ticks.step().expect("linear ticks have a uniform step");
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

#[test]
fn deterministic_extreme_ranges_remain_finite_ascending_and_bounded() {
    let min_subnormal = f64::from_bits(1);
    let next_after_one = f64::from_bits(1.0f64.to_bits() + 1);
    let ranges = [
        (-f64::MAX, f64::MAX),
        (f64::MIN_POSITIVE, f64::MAX),
        (-1e300, 1e300),
        (-f64::MIN_POSITIVE, f64::MIN_POSITIVE),
        (-min_subnormal, min_subnormal),
        (0.0, min_subnormal),
        (1.0, next_after_one),
    ];
    for (lo, hi) in ranges {
        for target in [0, 2, 8, 10_000] {
            let ticks = Ticks::linear(lo, hi, target);
            assert!((1..=200).contains(&ticks.len()), "[{lo}, {hi}], {target}");
            assert!(
                ticks.iter().all(|tick| tick.value.is_finite()),
                "[{lo}, {hi}], {target}"
            );
            assert!(
                ticks
                    .as_slice()
                    .windows(2)
                    .all(|pair| pair[0].value < pair[1].value),
                "[{lo}, {hi}], {target}"
            );
        }
    }
}

#[test]
fn fallback_ticks_are_formatted_at_the_shared_budget_never_by_display() {
    // Equal bounds: one tick, the set formatter's label — never `-0` or a
    // float artifact.
    assert_eq!(labels(&Ticks::linear(-0.0, -0.0, 5)), ["0"]);
    assert_eq!(labels(&Ticks::linear(1e-7, 1e-7, 5)), ["100.0n"]);
    assert_eq!(labels(&Ticks::linear(0.1 + 0.2, 0.1 + 0.2, 5)), ["0.3000"]);
    // A span that overflows: the bounds in exponent form, not 309 digits.
    let extreme = Ticks::linear(-f64::MAX, f64::MAX, 6);
    assert_eq!(labels(&extreme), ["-1.797e308", "1.797e308"]);
    // Beyond the prefix table the search's own ticks share one power of ten,
    // exact and short, instead of a hundred zeros under a clamped prefix.
    let tiny = Ticks::linear(8.796369082575082e-100, 8.796369082575112e-100, 5);
    for tick in &tiny {
        assert!(tick.label.ends_with("e-100"), "{}", tick.label);
        assert!(tick.label.len() < 24, "{}", tick.label);
        let parsed: f64 = tick.label.parse().expect("exponent labels parse");
        assert_eq!(parsed, tick.value, "{}", tick.label);
    }
    // Two distinct bounds that the budget would merge widen until they differ.
    let narrow = super::endpoint_ticks(1e15, 1e15 + 1.0);
    assert_ne!(narrow[0].label, narrow[1].label);
    for tick in &narrow {
        let parsed: f64 = tick.label.parse().expect("exponent labels parse");
        assert_eq!(parsed, tick.value, "{}", tick.label);
    }
}

/// The value of one unit in a label's last digit: `1.234e5` resolves to
/// `1e2`, `0.50` to `0.01`, `7` to `1`.
fn resolution(numeric: &str) -> f64 {
    let (mantissa, exponent) = numeric.split_once('e').unwrap_or((numeric, "0"));
    let exponent: i32 = exponent.parse().unwrap();
    let fraction = mantissa
        .find('.')
        .map_or(0, |point| mantissa.len() - point - 1) as i32;
    // Parsed, not `powi`: a chain of multiplications loses digits in the
    // subnormal range, where the sweep also goes.
    format!("1e{}", exponent - fraction).parse().unwrap()
}

/// A deterministic xorshift generator, so the sweep below is reproducible
/// without a dependency.
struct Xorshift(u64);

impl Xorshift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

#[test]
fn a_random_sweep_over_every_magnitude_keeps_the_label_contract() {
    let mut random = Xorshift(0x9E37_79B9_7F4A_7C15);
    for case in 0..20_000 {
        // Log-uniform bounds across the whole finite range, signed, with a
        // span that ranges from the ulp scale to the full magnitude.
        let magnitude = random.unit() * 600.0 - 300.0;
        let lo = 10f64.powf(magnitude)
            * if random.next().is_multiple_of(2) {
                1.0
            } else {
                -1.0
            };
        let span = 10f64.powf(magnitude - random.unit() * 20.0);
        let hi = if case % 7 == 0 { lo } else { lo + span };
        if !(lo.is_finite() && hi.is_finite()) {
            continue;
        }
        let target = 2 + (random.next() % 12) as usize;
        let ticks = Ticks::linear(lo, hi, target);
        assert!((1..=200).contains(&ticks.len()), "[{lo}, {hi}] × {target}");
        let values: Vec<f64> = ticks.iter().map(|tick| tick.value).collect();
        assert!(values.iter().all(|v| v.is_finite()), "[{lo}, {hi}]");
        assert!(
            values.windows(2).all(|pair| pair[0] < pair[1]),
            "not ascending in [{lo}, {hi}]: {values:?}"
        );
        let all_labels = labels(&ticks);
        for (index, label) in all_labels.iter().enumerate() {
            assert!(!label.is_empty(), "[{lo}, {hi}]");
            assert_ne!(*label, "-0", "[{lo}, {hi}]");
            assert!(!label.starts_with("-0.0") || values[index] != 0.0);
            assert!(
                !label.contains("00000000000"),
                "a Display-style label leaked in [{lo}, {hi}]: {label}"
            );
            // Every label decodes to its value within the budget it was
            // written at (exactly, for the search's ticks).
            let (numeric, factor) = decode(label);
            let parsed = decoded(label);
            let value = values[index];
            // The search's ticks decode exactly; an endpoint fallback (two
            // bounds, or one) decodes within half a unit of its last digit.
            let tolerance = if ticks.len() > 2 {
                0.0
            } else {
                // Half a unit of the last digit, plus the ulp the parsed
                // label and the value may each sit on.
                0.5 * resolution(numeric) * factor + 2.0 * (value.next_up() - value).abs()
            };
            assert!(
                (parsed - value).abs() <= tolerance,
                "label {label} decodes to {parsed} for value {value} in [{lo}, {hi}]"
            );
        }
        // Distinct ticks carry distinct labels.
        let mut sorted = all_labels.clone();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            all_labels.len(),
            "[{lo}, {hi}]: {all_labels:?}"
        );
    }
}

#[test]
fn strided_decades_prefer_multiples_of_the_stride() {
    // Nine decades at a target of three: stride 3, and the phase on multiples
    // of three keeps as many ticks as any other, so it wins.
    let ticks = Ticks::log10(1.0, 1e8, 3);
    assert_eq!(labels(&ticks), ["1", "10\u{00B3}", "10\u{2076}"]);
    // Eight decades from 10¹: the aligned phase would keep only 10³ and 10⁶,
    // so the phase with three ticks wins instead.
    let ticks = Ticks::log10(10.0, 1e8, 3);
    assert_eq!(labels(&ticks), ["10", "10\u{2074}", "10\u{2077}"]);
    // No stride: every decade, unchanged.
    let ticks = Ticks::log10(1.0, 1e4, 8);
    assert_eq!(ticks.len(), 5);
}

#[test]
fn integer_ticks_never_step_below_one() {
    let options = TickOptions::new().integer();
    assert_eq!(
        labels(&Ticks::linear_with(0.0, 3.0, 12, &options)),
        ["0", "1", "2", "3"]
    );
    assert_eq!(
        labels(&Ticks::linear_with(0.0, 1.0, 8, &options)),
        ["0", "1"]
    );
    // Large ranges are untouched: their steps were whole already.
    assert_eq!(
        Ticks::linear_with(0.0, 100.0, 6, &options),
        Ticks::linear(0.0, 100.0, 6)
    );
}

#[test]
fn units_label_the_axis_and_keep_the_values() {
    let si = TickOptions::new().unit(Unit::si("B"));
    let ticks = Ticks::linear_with(0.0, 50_000.0, 6, &si);
    assert_eq!(
        labels(&ticks),
        ["0 kB", "10 kB", "20 kB", "30 kB", "40 kB", "50 kB"]
    );
    assert_eq!(ticks.as_slice()[1].value, 10_000.0);
    let small = Ticks::linear_with(0.0, 12.0, 4, &si);
    assert_eq!(labels(&small), ["0 B", "4 B", "8 B", "12 B"]);

    let suffix = TickOptions::new().unit(Unit::suffix("%"));
    assert_eq!(
        labels(&Ticks::linear_with(0.0, 100.0, 6, &suffix)),
        ["0%", "20%", "40%", "60%", "80%", "100%"]
    );
    // A suffix never takes an SI prefix, however large the axis.
    let big = Ticks::linear_with(0.0, 50_000.0, 6, &suffix);
    assert!(
        big.iter().all(|t| !t.label.contains('k')),
        "{:?}",
        labels(&big)
    );

    let bytes = TickOptions::new().unit(Unit::Bytes);
    let ticks = Ticks::linear_with(0.0, 1_200_000.0, 6, &bytes);
    assert!(
        ticks.iter().all(|t| t.label.ends_with(" MiB")),
        "{:?}",
        labels(&ticks)
    );
    // Ticks are nice in MiB and exact in bytes: 0.2 MiB is 209715.2 B.
    let fifth = ticks
        .iter()
        .find(|t| t.label.starts_with("0.2"))
        .expect("a fifth-of-a-MiB tick");
    assert_eq!(fifth.value, 0.2 * 1_048_576.0);
    let tiny = Ticks::linear_with(0.0, 800.0, 5, &bytes);
    assert!(
        tiny.iter().all(|t| t.label.ends_with(" B")),
        "{:?}",
        labels(&tiny)
    );
    // Equal bounds carry the unit too.
    assert_eq!(
        labels(&Ticks::linear_with(2048.0, 2048.0, 3, &bytes)),
        ["2 KiB"]
    );
    assert_eq!(
        labels(&Ticks::linear_with(25_000.0, 25_000.0, 3, &si)),
        ["25.00 kB"]
    );
}
