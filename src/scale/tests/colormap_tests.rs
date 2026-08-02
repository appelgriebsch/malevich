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
