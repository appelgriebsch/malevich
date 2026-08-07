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
fn bands_cells_and_log_scales_round_trip_as_valid_specs() {
    let plots = [
        Plot::new()
            .layer(Bars::new(["a", "b", "c"], &[3.0, 7.0, 5.0][..]))
            .x_scale(Scale::bands(["a", "b", "c"])),
        Plot::new()
            .layer(Cells::matrix(2, &[1.0, 2.0, 3.0, 4.0][..]).extents((1.0, 100.0), (1.0, 1000.0)))
            .x_scale(Scale::Log)
            .y_scale(Scale::Log),
    ];
    for plot in plots {
        assert!(plot.validate().is_ok());
        let encoded = serde_json::to_string(&plot).expect("serializes");
        let decoded: Plot = serde_json::from_str(&encoded).expect("deserializes");
        assert!(decoded.validate().is_ok());
        assert_eq!(plot.render(&frame()), decoded.render(&frame()));
    }
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
fn malformed_payloads_render_without_panicking() {
    // Deserialization can produce states the constructors forbid; rendering must
    // shed them, never panic (COR-04).
    let colormap: crate::scale::Colormap =
        serde_json::from_str(r#"{"stops":[]}"#).expect("empty colormap deserializes");
    assert_eq!(colormap.color(0.5), Color::Default);

    let grid: Grid = serde_json::from_str(
        r#"{"columns":0,"plots":[{"layers":[],"title":null,"x":"Linear","y":"Linear","x_label":null,"y_label":null,"x_domain":null,"y_domain":null}]}"#,
    )
    .expect("zero-column grid deserializes");
    let _ = grid.render(&frame());

    // A Range with ragged x/low/high/marker channels inside a plot.
    let ragged = r#"{"layers":[{"Range":{"placement":{"Numeric":[0.0,1.0,2.0]},"low":[0.0],"high":[5.0,6.0],"body":null,"marker":[1.0,2.0,3.0,4.0],"color":null,"label":null}}],"title":null,"x":"Linear","y":"Linear","x_label":null,"y_label":null,"x_domain":null,"y_domain":null}"#;
    let plot: Plot = serde_json::from_str(ragged).expect("ragged range deserializes");
    let _ = plot.render(&frame());
}

#[test]
fn validate_rejects_the_malformed_payloads_render_tolerates() {
    // The strict boundary reports what the lenient renderer sheds.
    let ragged: Plot = serde_json::from_str(
        r#"{"layers":[{"Cells":{"columns":0,"values":[1.0,2.0,3.0],"extents":null,"colormap":{"stops":[[0,0,0],[255,255,255]]}}}],"title":null,"x":"Linear","y":"Linear","x_label":null,"y_label":null,"x_domain":null,"y_domain":null}"#,
    )
    .expect("zero-column cells deserializes");
    assert!(matches!(
        ragged.validate(),
        Err(crate::Error::EmptyDimension { .. })
    ));
    assert!(ragged.try_render(&frame()).is_err());
    // Rendering the same spec still does not panic.
    let _ = ragged.render(&frame());

    let degenerate: Plot = serde_json::from_str(
        r#"{"layers":[{"Cells":{"columns":1,"values":[1.0],"extents":[[2.0,2.0],[0.0,1.0]],"colormap":{"stops":[[0,0,0],[255,255,255]]}}}],"title":null,"x":"Linear","y":"Linear","x_label":null,"y_label":null,"x_domain":null,"y_domain":null}"#,
    )
    .expect("degenerate Cells extents deserialize");
    assert!(matches!(
        degenerate.validate(),
        Err(crate::Error::InvalidParameter { .. })
    ));

    let categorical_y: Plot = serde_json::from_str(
        r#"{"layers":[],"title":null,"x":"Linear","y":{"Bands":["a"]},"x_label":null,"y_label":null,"x_domain":null,"y_domain":null}"#,
    )
    .expect("categorical y scale deserializes");
    assert!(matches!(
        categorical_y.validate(),
        Err(crate::Error::IncompatibleScale { .. })
    ));
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
