use super::{Bins, Normalization, binned};
use crate::stat::Reducer;

#[test]
fn values_land_in_their_bins_and_the_last_edge_is_inclusive() {
    let mut bins = Bins::new(0.0, 1.0, 4);
    for value in [0.0, 0.5, 1.0, 2.5, 3.9, 4.0] {
        bins.add(value);
    }
    assert_eq!(bins.counts(), [2, 1, 1, 2]);
}

#[test]
fn out_of_range_and_gap_values_are_ignored() {
    let mut bins = Bins::new(0.0, 1.0, 2);
    for value in [-0.1, 2.1, f64::NAN, 0.5] {
        bins.add(value);
    }
    assert_eq!(bins.counts(), [1, 0]);
}

#[test]
fn merged_chunks_equal_one_sequential_pass() {
    let values: Vec<f64> = (0..5_000).map(|i| ((i * 37) % 100) as f64 / 10.0).collect();
    let mut sequential = Bins::new(0.0, 1.0, 10);
    for &value in &values {
        sequential.add(value);
    }
    let mut merged = Bins::new(0.0, 1.0, 10);
    for chunk in values.chunks(613) {
        let mut partial = Bins::new(0.0, 1.0, 10);
        for &value in chunk {
            partial.add(value);
        }
        merged.merge(&partial);
    }
    assert_eq!(sequential, merged);
}

#[test]
fn auto_bins_cover_the_data_with_nice_edges() {
    let values: Vec<f64> = (0..1_000)
        .map(|i| ((i * 61) % 997) as f64 / 100.0)
        .collect();
    let bins = Bins::auto(&values, 60).unwrap();
    assert!(bins.start() <= 0.0);
    assert!(bins.end() >= 9.96);
    assert_eq!(bins.counts().iter().sum::<u64>(), 1_000);
    // Nice-decimal width: a short exact decimal.
    let width = format!("{}", bins.width());
    assert!(width.len() <= 5, "width {width} is not a nice decimal");
}

#[test]
fn constant_data_gets_one_bin() {
    let bins = Bins::auto(&[7.0; 42], 60).unwrap();
    assert_eq!(bins.counts(), [42]);

    let extreme = Bins::auto(&[f64::MAX; 2], 60).unwrap();
    assert_eq!(extreme.counts(), [2]);
    assert!(extreme.start().is_finite() && extreme.width().is_finite());
    assert!(extreme.end().is_finite() && extreme.start() < extreme.end());
}

#[test]
fn no_finite_data_means_no_bins() {
    assert!(Bins::auto(&[f64::NAN], 60).is_none());
    assert!(Bins::auto(&[], 60).is_none());
}

#[test]
fn auto_never_drops_finite_values_and_respects_the_cap() {
    for offset in [0.0, 1e6, 1e12] {
        for span in [1e-3, 1.0, 1e6] {
            let values: Vec<f64> = (0..101).map(|i| offset + span * i as f64 / 100.0).collect();
            for limit in [1usize, 2, 3, 7, 60] {
                let bins = super::Bins::auto(&values, limit).expect("finite data bins");
                let sum: u64 = bins.counts().iter().sum();
                assert_eq!(
                    sum, 101,
                    "offset {offset} span {span} limit {limit} dropped data"
                );
                assert!(bins.counts().len() <= limit.max(1), "exceeded the cap");
            }
        }
    }
}

#[test]
fn auto_bins_cover_opposite_finite_extremes_without_panicking() {
    let bins = Bins::try_auto(&[-f64::MAX, f64::MAX], 60).unwrap().unwrap();
    assert_eq!(bins.counts().iter().sum::<u64>(), 2);
    assert!(bins.start().is_finite());
    assert!(bins.width().is_finite() && bins.width() > 0.0);
    assert!(bins.end().is_finite() && bins.end() >= f64::MAX);

    assert!(matches!(
        Bins::try_auto(&[-f64::MAX, f64::MAX], 1),
        Err(crate::Error::InvalidParameter { .. })
    ));
}

#[test]
fn bins2_of_constant_data_keeps_a_drawable_extent() {
    let grid = super::bins2(&[3.0, 3.0, 3.0], &[7.0, 7.0, 7.0], 8, 8).expect("finite pairs");
    assert!(grid.x.0 < grid.x.1, "x extent must be drawable");
    assert!(grid.y.0 < grid.y.1, "y extent must be drawable");
    assert_eq!(grid.counts.iter().sum::<f64>(), 3.0);
}

#[test]
fn bins2_distinguishes_opposite_finite_extremes() {
    let grid = super::try_bins2(&[-f64::MAX, f64::MAX], &[0.0, 0.0], 2, 1)
        .unwrap()
        .unwrap();
    assert_eq!(grid.counts, [1.0, 1.0]);
    assert_eq!(grid.x, (-f64::MAX, f64::MAX));
}

#[test]
fn caller_selected_histogram_geometry_is_bounded() {
    assert!(matches!(
        Bins::try_new(0.0, 1.0, usize::MAX),
        Err(crate::Error::DimensionTooLarge { .. })
    ));
    assert!(matches!(
        super::try_bins2(&[1.0], &[1.0], usize::MAX, 2),
        Err(crate::Error::DimensionTooLarge { .. })
    ));
}

#[test]
fn uniform_bins_cover_the_requested_extent_and_include_the_last_edge() {
    let bins = Bins::try_uniform(&[0.0, 0.25, 0.5, 0.75, 1.0], 2)
        .unwrap()
        .unwrap();
    assert_eq!(bins.start(), 0.0);
    assert_eq!(bins.width(), 0.5);
    assert_eq!(bins.counts(), [2, 3]);
}

#[test]
fn uniform_bins_handle_constant_and_opposite_extreme_samples() {
    let constant = Bins::try_uniform(&[f64::MAX; 3], 4).unwrap().unwrap();
    assert_eq!(constant.counts().len(), 4);
    assert_eq!(constant.counts().iter().sum::<u64>(), 3);

    let extremes = Bins::try_uniform(&[-f64::MAX, f64::MAX], 2)
        .unwrap()
        .unwrap();
    assert_eq!(extremes.counts(), [1, 1]);
    assert!(matches!(
        Bins::try_uniform(&[-f64::MAX, f64::MAX], 1),
        Err(crate::Error::InvalidParameter { .. })
    ));
}

#[test]
fn uniform_bins_validate_the_count_even_without_finite_data() {
    assert!(Bins::try_uniform(&[f64::NAN], 2).unwrap().is_none());
    assert!(matches!(
        Bins::try_uniform(&[], 0),
        Err(crate::Error::EmptyDimension { .. })
    ));
    assert!(matches!(
        Bins::try_uniform(&[], usize::MAX),
        Err(crate::Error::DimensionTooLarge { .. })
    ));
}

#[test]
fn accepted_uniform_geometry_never_drops_its_finite_endpoints() {
    // Deterministic bit-pattern coverage across signs, exponents, and adjacent
    // values. Some one-bin spans are mathematically wider than f64 and correctly
    // return an error; every accepted geometry must retain both endpoints.
    let mut state = 0x6a09_e667_f3bc_c909_u64;
    for _ in 0..2_000 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let first = f64::from_bits(state);
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let second = f64::from_bits(state);
        if !(first.is_finite() && second.is_finite() && first != second) {
            continue;
        }
        for count in [1, 2, 3, 7, 31] {
            if let Ok(Some(bins)) = Bins::try_uniform(&[first, second], count) {
                assert_eq!(
                    bins.counts().iter().sum::<u64>(),
                    2,
                    "dropped an endpoint for {first:?}..{second:?} in {count} bins"
                );
            }
        }
    }
}

#[test]
fn binned_reducers_share_streaming_and_buffered_execution_semantics() {
    let bins = Bins::new(0.0, 1.0, 3);
    let x = [0.1, 0.2, 1.1, 1.2, f64::NAN];
    let y = [1.0, 3.0, 10.0, f64::NAN, 99.0];

    assert_eq!(binned(&x, &y, &bins, Reducer::Count), [2.0, 1.0, 0.0]);
    for reducer in [Reducer::Mean, Reducer::Median] {
        let reduced = binned(&x, &y, &bins, reducer);
        assert_eq!(&reduced[..2], [2.0, 10.0]);
        assert!(reduced[2].is_nan());
    }
}

#[test]
fn whole_number_data_gets_whole_widths_and_half_integer_edges() {
    // Ten integers: Sturges asks for five bins, the tick step would be 2.5,
    // and the whole step at or below it is 2 — two integers per bin, every
    // bin alike, the maximum not sharing the last bin with its neighbor.
    let digits: Vec<f64> = (0..10).map(f64::from).collect();
    let bins = Bins::auto(&digits, 60).unwrap();
    assert_eq!(bins.width(), 2.0);
    assert_eq!(bins.start(), -0.5);
    assert_eq!(bins.counts(), [2u64; 5]);

    // Ten thousand values in 0..=9: Freedman–Diaconis would ask for a
    // fractional width; whole data takes width 1, never 0.5 with empties.
    let dense: Vec<f64> = (0..10_000).map(|i| f64::from(i % 10)).collect();
    let bins = Bins::auto(&dense, 60).unwrap();
    assert_eq!(bins.width(), 1.0);
    assert_eq!(bins.counts(), [1000u64; 10]);

    // A wide integer span keeps a whole nice width and every integer in one bin.
    let wide: Vec<f64> = (0..1_000).map(|i| f64::from((i * 37) % 997)).collect();
    let bins = Bins::auto(&wide, 60).unwrap();
    assert_eq!(bins.width().fract(), 0.0, "width {}", bins.width());
    assert_eq!((bins.start() + 0.5).fract(), 0.0, "start {}", bins.start());
    assert_eq!(bins.counts().iter().sum::<u64>(), 1_000);
    // One to fifty at width ten: five bins of ten integers, none lonely.
    let fifty: Vec<f64> = (1..=50).map(f64::from).collect();
    let bins = Bins::auto(&fifty, 60).unwrap();
    assert_eq!((bins.start(), bins.width()), (0.5, 10.0));
    assert_eq!(bins.counts(), [10u64; 5]);
    assert_eq!(super::whole_nice_step(2.5), 2.0);
    assert_eq!(super::whole_nice_step(0.5), 1.0);
    assert_eq!(super::whole_nice_step(30.0), 20.0);
    let extreme = super::whole_nice_step(f64::MAX);
    assert!(extreme.is_finite() && extreme <= f64::MAX);
}

#[test]
fn freedman_diaconis_uses_type_7_quartiles() {
    // [1.1, 2.1, 3.1, 10.1]: type-7 IQR is 3 (raw order statistics would say
    // 8), so the FD width 2·3/4^(1/3) ≈ 3.8 asks for about three bins over
    // the span, not one.
    let bins = Bins::auto(&[1.1, 2.1, 3.1, 10.1], 60).unwrap();
    assert!(bins.counts().len() >= 3, "{:?}", bins.counts());
}

#[test]
fn heights_rescale_the_same_counts() {
    let mut bins = Bins::new(0.0, 0.5, 4);
    for value in [0.1, 0.2, 0.6, 0.7, 0.8, 1.2, 1.9, 1.95] {
        bins.add(value);
    }
    assert_eq!(bins.counts(), [2, 3, 1, 2]);
    assert_eq!(
        bins.heights(Normalization::Count, false),
        [2.0, 3.0, 1.0, 2.0]
    );
    assert_eq!(
        bins.heights(Normalization::Probability, false),
        [0.25, 0.375, 0.125, 0.25]
    );
    assert_eq!(
        bins.heights(Normalization::Percent, false),
        [25.0, 37.5, 12.5, 25.0]
    );
    // Density integrates to one: heights times the bin width sum to one.
    let density = bins.heights(Normalization::Density, false);
    assert_eq!(density, [0.5, 0.75, 0.25, 0.5]);
    assert_eq!(density.iter().map(|h| h * bins.width()).sum::<f64>(), 1.0);
}

#[test]
fn cumulative_heights_end_at_the_total_one_or_one_hundred() {
    let mut bins = Bins::new(0.0, 0.5, 4);
    for value in [0.1, 0.2, 0.6, 0.7, 0.8, 1.2, 1.9, 1.95] {
        bins.add(value);
    }
    assert_eq!(
        bins.heights(Normalization::Count, true),
        [2.0, 5.0, 6.0, 8.0]
    );
    assert_eq!(
        bins.heights(Normalization::Probability, true),
        [0.25, 0.625, 0.75, 1.0]
    );
    assert_eq!(
        bins.heights(Normalization::Percent, true),
        [25.0, 62.5, 75.0, 100.0]
    );
    // A cumulative density is the distribution function, not a running
    // sum of densities: it ends at one whatever the bin width.
    assert_eq!(
        bins.heights(Normalization::Density, true),
        [0.25, 0.625, 0.75, 1.0]
    );
}

#[test]
fn an_empty_histogram_normalizes_to_zeros() {
    let bins = Bins::new(0.0, 1.0, 3);
    for normalization in [
        Normalization::Count,
        Normalization::Probability,
        Normalization::Percent,
        Normalization::Density,
    ] {
        for cumulative in [false, true] {
            assert_eq!(bins.heights(normalization, cumulative), [0.0, 0.0, 0.0]);
        }
    }
}
