use super::Grid;
use crate::plot::Frame;
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
        let visible: usize = {
            let mut width = 0usize;
            let mut chars = line.chars();
            while let Some(c) = chars.next() {
                if c == '\u{1b}' {
                    for f in chars.by_ref() {
                        if f.is_ascii_alphabetic() {
                            break;
                        }
                    }
                    continue;
                }
                width += 1;
            }
            width
        };
        assert!(visible <= 50, "line overflows: {line:?}");
    }
}

#[test]
fn empty_grids_render_nothing() {
    assert_eq!(Grid::new(3).render(&Frame::plain(40, 10)), "");
}
