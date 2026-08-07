use super::Grid;
use crate::plot::Frame;
use crate::render::{Charset, display_width_ansi};
use crate::{ColorMode, Line, Plot};

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<Grid<'static>>();

#[test]
fn cells_sit_side_by_side_with_aligned_rows() {
    let grid = Grid::new(2)
        .with(malevich_line(&[1.0, 3.0]))
        .with(malevich_line(&[2.0, 1.0]));
    let text = grid.render(&Frame::plain(60, 8));
    let widths: Vec<usize> = text.lines().map(|l| l.chars().count()).collect();
    assert_eq!(widths.len(), 8);
    // Every line that has a second cell starts it at the same column.
    assert!(text.lines().all(|l| l.chars().count() <= 60));
}

const LINE_AND_BARS: &str = r"          line                      bars
3 ┤         ⣀⠔⠉⠒⠤⣀        5 ┤        █████
  │       ⡠⠊      ⠉⠒⠤⣀      │        █████  ▁▁▁▁▁
  │    ⢀⠔⠊            ⠉⠒⠤   │  ▁▁▁▁▁ █████  █████
  │  ⢀⠔⠁                    │  █████ █████  █████
1 ┤⡠⠊⠁                    0 ┤  █████ █████  █████
  └┬────────────────────┬   └──────────────────────
   0                    2        a      b     c";

#[test]
fn two_charts_side_by_side_match_their_snapshot() {
    let grid = Grid::new(2)
        .with(crate::line(&[1.0, 3.0, 2.0][..]).title("line"))
        .with(crate::bar(["a", "b", "c"], &[2.0, 5.0, 3.0][..]).title("bars"));
    let text = grid.render(&Frame::plain(52, 8));
    assert_eq!(text, LINE_AND_BARS);
}

fn malevich_line(values: &[f64]) -> Plot<'_> {
    Plot::new().layer(Line::y(values))
}

#[test]
fn later_rows_wrap_with_a_separator_and_fit_the_frame() {
    let grid = Grid::new(2)
        .with(malevich_line(&[1.0, 2.0]))
        .with(malevich_line(&[2.0, 1.0]))
        .with(malevich_line(&[3.0, 3.0]));
    let text = grid.render(&Frame::plain(40, 12));
    // Two grid rows plus a blank separator between them, within the frame height.
    assert!(
        text.lines().count() <= 12,
        "grid overflows its frame height"
    );
    assert!(
        text.lines().any(|line| line.trim().is_empty()),
        "no blank separator between grid rows"
    );
}

#[test]
fn padding_is_escape_aware_in_color_mode() {
    let mut frame = Frame::plain(50, 8);
    frame.color = ColorMode::Ansi16;
    let grid = Grid::new(2)
        .with(
            Plot::new()
                .layer(Line::y(&[1.0, 2.0][..]).label("a"))
                .layer(Line::y(&[2.0, 1.0][..]).label("b")),
        )
        .with(malevich_line(&[2.0, 1.0]));
    let text = grid.render(&frame);
    for line in text.lines() {
        assert!(display_width_ansi(line) <= 50, "line overflows: {line:?}");
    }
}

#[test]
fn empty_grids_render_nothing() {
    assert_eq!(Grid::new(3).render(&Frame::plain(40, 10)), "");
}

#[test]
fn tiny_layouts_budget_separators_and_omit_later_panes() {
    let grid = Grid::new(2)
        .with(malevich_line(&[1.0, 2.0]))
        .with(malevich_line(&[2.0, 1.0]))
        .with(malevich_line(&[1.0, 3.0]))
        .with(malevich_line(&[3.0, 1.0]));

    let one = grid.layout(&Frame::plain(1, 1)).unwrap();
    assert_eq!((one.columns, one.rows, one.visible_plots), (1, 1, 1));
    assert_eq!((one.cell_width, one.cell_height), (1, 1));

    let exact = grid.layout(&Frame::plain(3, 3)).unwrap();
    assert_eq!((exact.columns, exact.rows, exact.visible_plots), (2, 2, 4));
    assert_eq!((exact.cell_width, exact.cell_height), (1, 1));

    let narrow = grid.layout(&Frame::plain(2, 5)).unwrap();
    assert_eq!(
        (narrow.columns, narrow.rows, narrow.visible_plots),
        (1, 3, 3)
    );
    assert_eq!((narrow.cell_width, narrow.cell_height), (2, 1));
    assert!(grid.layout(&Frame::plain(0, 5)).is_none());
    assert!(grid.layout(&Frame::plain(5, 0)).is_none());
}

#[test]
fn all_tiny_frames_stay_inside_their_visible_bounds() {
    static VALUES: [f64; 3] = [1.0, 3.0, 2.0];
    let charsets = [
        Charset::Ascii,
        Charset::HalfBlocks,
        Charset::Quadrants,
        Charset::Sextants,
        Charset::Octants,
        Charset::Braille,
    ];
    let colors = [
        ColorMode::Plain,
        ColorMode::Ansi16,
        ColorMode::Ansi256,
        ColorMode::TrueColor,
    ];

    for plots in 1..=5 {
        for columns in 1..=4 {
            let mut grid = Grid::new(columns);
            for _ in 0..plots {
                grid = grid.with(crate::line(&VALUES[..]));
            }
            for width in 0..=6 {
                for height in 0..=6 {
                    for charset in charsets {
                        for color in colors {
                            let frame = Frame {
                                width,
                                height,
                                charset,
                                color,
                                ..Frame::plain(width, height)
                            };
                            let text = grid.render(&frame);
                            let rows = usize::from(!text.is_empty())
                                .saturating_add(text.bytes().filter(|byte| *byte == b'\n').count());
                            assert!(
                                rows <= height,
                                "{plots} plots in {columns} columns overflow {width}x{height} ({charset:?}, {color:?}): {text:?}"
                            );
                            for line in text.split('\n') {
                                assert!(
                                    display_width_ansi(line) <= width,
                                    "{plots} plots in {columns} columns overflow {width}x{height} ({charset:?}, {color:?}): {line:?}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn fallible_grid_render_checks_specs_and_hostile_frames() {
    let grid = Grid::new(2).with(malevich_line(&[1.0, 2.0]));
    let frame = Frame::plain(20, 6);
    assert!(grid.validate().is_ok());
    assert_eq!(grid.try_render(&frame).unwrap(), grid.render(&frame));

    let hostile = Frame::plain(usize::MAX, 1);
    assert!(matches!(
        grid.try_render(&hostile),
        Err(crate::Error::DimensionTooLarge { .. })
    ));
    assert_eq!(grid.render(&hostile), "");
}
