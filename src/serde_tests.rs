use crate::render::{Charset, Color};
use crate::scale::Scale;
use crate::{Bars, Cells, Frame, Grid, Line, LineStyle, Plot, Points, Rule, Text};

fn frame() -> Frame {
    Frame {
        charset: Charset::Braille,
        ..Frame::plain(64, 18)
    }
}

#[test]
fn a_full_spec_round_trips_to_an_identical_render() {
    let plot = Plot::new()
        .layer(
            Line::xy(
                &[0.0, 1.0, 2.0, 3.0, 4.0][..],
                &[1.0, f64::NAN, 4.0, 2.0, 5.0][..],
            )
            .label("with a gap")
            .color(Color::Rgb(200, 40, 90))
            .style(LineStyle::Corners),
        )
        .layer(Points::xy(&[0.5, 2.5][..], &[3.0, 1.0][..]).label("dots"))
        .layer(Rule::h(2.5))
        .layer(Text::at(1.0, 4.5, "note"))
        .title("round trip")
        .x_label("x")
        .y_label("y")
        .y_domain(0.0, 6.0);

    let encoded = serde_json::to_string(&plot).expect("serializes");
    let decoded: Plot = serde_json::from_str(&encoded).expect("deserializes");
    assert_eq!(plot.render(&frame()), decoded.render(&frame()));
}

#[test]
fn gaps_survive_json_as_nulls() {
    let plot = Plot::new().layer(Line::y(&[1.0, f64::NAN, 3.0][..]));
    let encoded = serde_json::to_string(&plot).expect("serializes");
    assert!(encoded.contains("[1.0,null,3.0]"), "{encoded}");
    let decoded: Plot = serde_json::from_str(&encoded).expect("deserializes");
    assert_eq!(plot.render(&frame()), decoded.render(&frame()));
}

#[test]
fn bands_cells_and_log_scales_round_trip() {
    let plot = Plot::new()
        .layer(Bars::new(["a", "b", "c"], &[3.0, 7.0, 5.0][..]))
        .layer(Cells::matrix(2, &[1.0, 2.0, 3.0, 4.0][..]))
        .x_scale(Scale::bands(["a", "b", "c"]))
        .y_scale(Scale::Log);
    let encoded = serde_json::to_string(&plot).expect("serializes");
    let decoded: Plot = serde_json::from_str(&encoded).expect("deserializes");
    assert_eq!(plot.render(&frame()), decoded.render(&frame()));
}

#[test]
fn a_grid_of_plots_round_trips() {
    let grid = Grid::new(2)
        .with(crate::line(&[1.0, 3.0, 2.0][..]).title("a"))
        .with(crate::line(&[2.0, 1.0, 3.0][..]).title("b"));
    let encoded = serde_json::to_string(&grid).expect("serializes");
    let decoded: Grid = serde_json::from_str(&encoded).expect("deserializes");
    assert_eq!(grid.render(&frame()), decoded.render(&frame()));
}

#[test]
fn a_function_line_refuses_to_serialize() {
    let plot = Plot::new().layer(Line::function(0.0..10.0, f64::sin));
    let error = serde_json::to_string(&plot).expect_err("closures have no data form");
    assert!(
        error.to_string().contains("sample it into points"),
        "{error}"
    );
}
