use super::Colormap;
use crate::render::Color;

#[test]
fn endpoints_hit_the_terminal_stops() {
    let map = Colormap::DEFAULT;
    assert_eq!(map.color(0.0), Color::Rgb(68, 1, 84));
    assert_eq!(map.color(1.0), Color::Rgb(253, 231, 37));
}

#[test]
fn out_of_range_and_gap_positions_clamp() {
    let map = Colormap::DEFAULT;
    assert_eq!(map.color(-5.0), map.color(0.0));
    assert_eq!(map.color(5.0), map.color(1.0));
    assert_eq!(map.color(f64::NAN), map.color(0.0));
}

#[test]
fn midpoints_interpolate_between_stops() {
    let map = Colormap::new(&[(0, 0, 0), (100, 200, 50)]);
    assert_eq!(map.color(0.5), Color::Rgb(50, 100, 25));
}

#[test]
fn runtime_stops_move_into_an_owned_colormap() {
    let stops = vec![(3, 5, 8), (13, 21, 34), (55, 89, 144)];
    let allocation = stops.as_ptr();
    let map = Colormap::try_from_stops(stops).unwrap();

    assert_eq!(map.stops(), [(3, 5, 8), (13, 21, 34), (55, 89, 144)]);
    assert_eq!(map.stops().as_ptr(), allocation, "the vector was copied");
    assert_eq!(map.color(1.0), Color::Rgb(55, 89, 144));
}

#[test]
fn a_runtime_colormap_requires_two_stops() {
    for stops in [Vec::new(), vec![(1, 2, 3)]] {
        assert!(matches!(
            Colormap::try_from_stops(stops),
            Err(crate::Error::EmptyDimension {
                what: "Colormap stops"
            })
        ));
    }
}

#[test]
fn every_canonical_name_resolves_and_unknown_names_do_not() {
    for name in Colormap::NAMES {
        let map = Colormap::named(name).expect("canonical name failed to resolve");
        assert!(map.stops().len() >= 2, "{name} has too few stops");
        assert!(map.midpoint().is_none(), "{name} came back pre-anchored");
    }
    assert_eq!(Colormap::named("grays"), Colormap::named("greys"));
    assert_eq!(Colormap::named("jet"), None, "no rainbow maps");
    assert_eq!(Colormap::named("VIRIDIS"), None, "names are lowercase");
}

#[test]
fn the_default_map_is_viridis() {
    assert_eq!(Colormap::DEFAULT, Colormap::VIRIDIS);
    assert_eq!(Colormap::default(), Colormap::VIRIDIS);
}

#[test]
fn a_centered_map_pins_its_midpoint_to_the_ramp_middle() {
    let map = Colormap::RED_BLUE.centered_at(0.0);
    // Data on [-1, 0.5]: the larger side (1.0) sets the symmetric span [-1, 1].
    assert_eq!(map.position_in(0.0, -1.0, 0.5), 0.5);
    assert_eq!(map.position_in(-1.0, -1.0, 0.5), 0.0);
    assert_eq!(map.position_in(0.5, -1.0, 0.5), 0.75);
    // Equal magnitudes on either side get equal distances from the middle.
    let below = map.position_in(-0.3, -1.0, 0.5);
    let above = map.position_in(0.3, -1.0, 0.5);
    assert!((0.5 - below - (above - 0.5)).abs() < 1e-12);
}

#[test]
fn a_linear_map_spans_the_observed_range() {
    let map = Colormap::VIRIDIS;
    assert_eq!(map.position_in(2.0, 2.0, 6.0), 0.0);
    assert_eq!(map.position_in(6.0, 2.0, 6.0), 1.0);
    assert_eq!(map.position_in(4.0, 2.0, 6.0), 0.5);
    // Out-of-range and NaN degrade exactly like Colormap::color.
    assert_eq!(map.position_in(9.0, 2.0, 6.0), 1.0);
    assert_eq!(map.position_in(f64::NAN, 2.0, 6.0), 0.0);

    assert_eq!(map.position_in(-f64::MAX, -f64::MAX, f64::MAX), 0.0);
    assert_eq!(map.position_in(0.0, -f64::MAX, f64::MAX), 0.5);
    assert_eq!(map.position_in(f64::MAX, -f64::MAX, f64::MAX), 1.0);
}

#[test]
fn a_degenerate_range_centers_on_the_midpoint() {
    let map = Colormap::RED_BLUE.centered_at(1.0);
    assert_eq!(map.position_in(1.0, 1.0, 1.0), 0.5);
}

#[test]
#[should_panic(expected = "finite midpoint")]
fn centering_on_a_non_finite_value_is_misuse() {
    let _ = Colormap::RED_BLUE.centered_at(f64::NAN);
}

#[test]
fn a_deserialized_non_finite_midpoint_degrades_and_fails_validation() {
    // Unreachable through the constructors; only deserialization can build it.
    let map = Colormap {
        midpoint: Some(super::Exact(f64::NAN)),
        ..Colormap::RED_BLUE
    };
    assert!(map.validate().is_err());
    // Rendering paths degrade to the linear mapping instead of spreading NaN.
    assert_eq!(map.position_in(3.0, 2.0, 6.0), 0.25);
}

/// The curation criterion: every named map must stay distinguishable after the
/// honest quantizers, not just in truecolor. (The plain tier reads the ramp
/// position, not the color, so it is monotonic by construction.)
#[test]
fn named_maps_survive_the_color_ladder_distinguishably() {
    use crate::render::color::{rgb_to_16, rgb_to_256};

    let rgb = |color: Color| match color {
        Color::Rgb(r, g, b) => (r, g, b),
        other => panic!("named maps interpolate to concrete RGB, got {other:?}"),
    };
    for name in Colormap::NAMES {
        let map = Colormap::named(name).unwrap();
        let low = rgb(map.color(0.0));
        let mid = rgb(map.color(0.5));
        let high = rgb(map.color(1.0));
        for (quantize, tier) in [
            (rgb_to_256 as fn(u8, u8, u8) -> u8, "256"),
            (rgb_to_16, "16"),
        ] {
            let low = quantize(low.0, low.1, low.2);
            let high = quantize(high.0, high.1, high.2);
            let mid = quantize(mid.0, mid.1, mid.2);
            assert_ne!(low, high, "{name}: ends collapse at {tier} colors");
            // Diverging maps must also keep both ends apart from the neutral
            // middle, or the sign of the data disappears.
            if map == Colormap::RED_BLUE || map == Colormap::PURPLE_ORANGE {
                assert_ne!(
                    low, mid,
                    "{name}: low end collapses into the middle at {tier}"
                );
                assert_ne!(
                    high, mid,
                    "{name}: high end collapses into the middle at {tier}"
                );
            }
        }
    }
}

#[test]
fn log_ramps_position_by_decade() {
    let map = Colormap::GREYS.log();
    assert!(map.is_log());
    let position = map.position_in(10.0, 1.0, 1000.0);
    assert!((position - 1.0 / 3.0).abs() < 1e-12, "got {position}");
    assert_eq!(map.position_in(1.0, 1.0, 1000.0), 0.0);
    assert_eq!(map.position_in(1000.0, 1.0, 1000.0), 1.0);
    // The same value on the linear ramp sits at the very bottom — the
    // collapse log() exists to prevent.
    assert!(Colormap::GREYS.position_in(10.0, 1.0, 1000.0) < 0.01);
}

#[test]
fn log_ramps_gap_nonpositive_values_and_ranges() {
    let map = Colormap::GREYS.log();
    assert!(map.position_in(0.0, 1.0, 1000.0).is_nan());
    assert!(map.position_in(-5.0, 1.0, 1000.0).is_nan());
    assert!(map.position_in(f64::NAN, 1.0, 1000.0).is_nan());
    // A non-positive observed range has no ramp at all.
    assert!(map.position_in(5.0, -1.0, 1000.0).is_nan());
    assert!(map.position_in(5.0, 0.0, 0.0).is_nan());
    // Degenerate positive range: everything at the low end, like linear.
    assert_eq!(map.position_in(7.0, 7.0, 7.0), 0.0);
}

#[test]
fn centered_and_log_together_fail_validation() {
    let map = Colormap::RED_BLUE.centered_at(0.0).log();
    assert!(matches!(
        map.validate(),
        Err(crate::Error::InvalidParameter { .. })
    ));
    assert!(Colormap::MAGMA.log().validate().is_ok());
}

#[test]
fn a_fixed_domain_replaces_the_observed_extent_and_discloses_the_outside() {
    let map = Colormap::GREYS.domain(0.0, 10.0);
    assert_eq!(map.fixed_domain(), Some((0.0, 10.0)));
    assert_eq!(map.display_domain(0.0, 100.0), (0.0, 10.0));
    assert_eq!(map.position_in(5.0, 0.0, 100.0), 0.5);
    // Outside the range: clamped without caps, disclosed with them.
    assert_eq!(map.sample(50.0, 0.0, 100.0), Some((1.0, map.color(1.0))));
    let capped = map.clone().under(Color::Blue).over(Color::Red);
    assert_eq!(capped.sample(-1.0, 0.0, 100.0), Some((0.0, Color::Blue)));
    assert_eq!(capped.sample(50.0, 0.0, 100.0), Some((1.0, Color::Red)));
    assert_eq!(capped.sample(5.0, 0.0, 100.0), Some((0.5, map.color(0.5))));
    assert_eq!(capped.sample(f64::NAN, 0.0, 100.0), None);
    // A log map needs a positive domain.
    assert!(Colormap::GREYS.log().domain(0.0, 1.0).validate().is_err());
    assert!(Colormap::GREYS.log().domain(1.0, 100.0).validate().is_ok());
}

#[test]
fn stepped_and_thresholded_maps_color_by_band() {
    let stepped = Colormap::GREYS.steps(4);
    assert_eq!(stepped.bands(), Some(4));
    // Every value in a band shares the band's center color.
    assert_eq!(stepped.sample(0.1, 0.0, 1.0), stepped.sample(0.2, 0.0, 1.0));
    assert_eq!(stepped.sample(0.1, 0.0, 1.0).unwrap().0, 0.125);
    assert_eq!(stepped.sample(1.0, 0.0, 1.0).unwrap().0, 0.875);
    assert_ne!(stepped.sample(0.2, 0.0, 1.0), stepped.sample(0.3, 0.0, 1.0));
    assert_eq!(stepped.boundaries(0.0, 1.0), [0.0, 0.25, 0.5, 0.75, 1.0]);

    let split = Colormap::GREYS.thresholds([3.0, 1.0, 1.0]);
    assert_eq!(split.bands(), Some(3));
    assert_eq!(split.boundaries(0.0, 4.0), [0.0, 1.0, 3.0, 4.0]);
    assert_eq!(split.sample(0.5, 0.0, 4.0).unwrap().0, 0.5 / 3.0);
    assert_eq!(split.sample(2.0, 0.0, 4.0).unwrap().0, 1.5 / 3.0);
    assert_eq!(split.sample(3.5, 0.0, 4.0).unwrap().0, 2.5 / 3.0);
    // Steps and thresholds replace each other; a continuous ramp is untouched.
    assert_eq!(split.clone().steps(2).bands(), Some(2));
    assert_eq!(Colormap::GREYS.bands(), None);
    assert_eq!(Colormap::GREYS.boundaries(0.0, 1.0), Vec::<f64>::new());
    assert_eq!(Colormap::GREYS.sample(0.3, 0.0, 1.0).unwrap().0, 0.3);
    // A stepped log ramp divides by decade.
    let decades = Colormap::GREYS.log().steps(2);
    assert_eq!(decades.boundaries(1.0, 100.0), [1.0, 10.0, 100.0]);
}
