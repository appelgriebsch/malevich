use super::{Chart, PrepareError, ValueMark, prepare};
use crate::args::{Args, Outcome, parse_from};
use crate::input;

fn args(arguments: &[&str]) -> Args {
    match parse_from(lexopt::Parser::from_args(arguments.iter().copied())) {
        Ok(Outcome::Run(args)) => *args,
        other => panic!("expected a run, got {other:?}"),
    }
}

#[test]
fn projection_and_column_roles_are_resolved_in_the_recipe() {
    let args = args(&["line", "-H", "--cols", "step,loss,score", "--fmt", "xyy"]);
    let table = input::frame(
        "step score loss ignored\n1 0.8 4 x\n2 0.9 3 y\n",
        None,
        true,
    );
    let recipe = prepare(&args, table).unwrap();

    let Chart::Value { mark, data } = recipe.chart else {
        panic!("expected value data")
    };
    assert_eq!(mark, ValueMark::Line);
    assert_eq!(data.series.len(), 2);
    assert_eq!(data.x(&data.series[0]), Some(&[1.0, 2.0][..]));
    assert_eq!(data.y(&data.series[0]), [4.0, 3.0]);
    assert_eq!(data.series[0].label.as_deref(), Some("loss"));
    assert_eq!(data.y(&data.series[1]), [0.8, 0.9]);
    assert_eq!(data.series[1].label.as_deref(), Some("score"));
}

#[test]
fn grouping_is_extracted_before_scatter_parsing() {
    let args = args(&["scatter", "-H", "--by", "kind"]);
    let table = input::frame("x kind y\n1 a 2\n3 b 4\n", None, true);
    let recipe = prepare(&args, table).unwrap();

    let Chart::ScatterBy { x, y, groups } = recipe.chart else {
        panic!("expected grouped scatter data")
    };
    assert_eq!(x, [1.0, 3.0]);
    assert_eq!(y, [2.0, 4.0]);
    assert_eq!(groups, ["a", "b"]);
}

#[test]
fn automatic_histogram_geometry_is_prepared_once() {
    let args = args(&["hist"]);
    let table = input::frame("1\n2\n2\n3\nbad\n", None, false);
    let recipe = prepare(&args, table).unwrap();

    let Chart::Histogram { heights, .. } = recipe.chart else {
        panic!("expected histogram geometry")
    };
    assert_eq!(heights.iter().sum::<f64>(), 4.0);
    assert_eq!(recipe.unparsed, 1);
}

#[test]
fn normalized_histograms_scale_through_the_same_bins() {
    let args = args(&["hist", "--normalize", "percent", "--cumulative"]);
    let table = input::frame("1\n2\n2\n3\n", None, false);
    let recipe = prepare(&args, table).unwrap();

    let Chart::Histogram {
        heights,
        normalization,
        ..
    } = recipe.chart
    else {
        panic!("expected histogram geometry")
    };
    assert_eq!(normalization, malevich::stat::Normalization::Percent);
    assert_eq!(heights.last(), Some(&100.0));
    assert!(heights.windows(2).all(|pair| pair[0] <= pair[1]));
}

#[test]
fn selector_failures_remain_input_errors() {
    let args = args(&["line", "--cols", "4"]);
    let table = input::frame("1 2\n", None, false);
    let error = prepare(&args, table).unwrap_err();

    assert!(matches!(error, PrepareError::Input(_)));
    assert!(error.to_string().contains("column index 4"));
}

#[test]
fn stacked_bars_carry_one_series_per_value_column() {
    let args = args(&["bar", "-H", "--stack"]);
    let table = input::frame("q a b\nQ1 3 4\nQ2 5 x\n", None, true);
    let recipe = prepare(&args, table).unwrap();
    let Chart::BarGroups {
        labels,
        names,
        series,
        layout,
        horizontal,
    } = recipe.chart
    else {
        panic!("expected bar groups")
    };
    assert_eq!(labels, ["Q1", "Q2"]);
    assert_eq!(names, ["a", "b"]);
    assert_eq!(series[0], [3.0, 5.0]);
    assert!(series[1][1].is_nan(), "an unparsable field is a gap");
    assert_eq!(layout, crate::args::BarLayout::Stack);
    assert!(!horizontal);
    assert_eq!(recipe.unparsed, 1);
}

#[test]
fn a_fixed_bin_width_starts_on_a_multiple_of_itself() {
    let args = args(&["hist", "--binwidth", "10"]);
    let table = input::frame("3\n12\n25\n33\n47\n", None, false);
    let recipe = prepare(&args, table).unwrap();
    let Chart::Histogram {
        start,
        width,
        heights,
        ..
    } = recipe.chart
    else {
        panic!("expected histogram geometry")
    };
    assert_eq!((start, width), (0.0, 10.0));
    assert_eq!(heights, [1.0, 1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn tables_name_their_rows_from_a_textual_first_column() {
    let args = args(&["table"]);
    let named = prepare(
        &args,
        input::frame("small 1 2\nlarge 30 400\n", None, false),
    )
    .unwrap();
    let Chart::Table {
        rows,
        columns,
        values,
    } = named.chart
    else {
        panic!("expected a table")
    };
    assert_eq!(rows, ["small", "large"]);
    assert_eq!(columns, ["1", "2"]);
    assert_eq!(values, [1.0, 2.0, 30.0, 400.0]);
    let numbered = prepare(&args, input::frame("1 2\n3 4\n", None, false)).unwrap();
    let Chart::Table { rows, .. } = numbered.chart else {
        panic!("expected a table")
    };
    assert_eq!(rows, ["1", "2"]);
}
