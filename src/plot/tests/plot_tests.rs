use super::Plot;
use crate::mark::Line;
use crate::plot::Frame;

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<Plot<'static>>();

#[test]
fn the_line_preset_equals_its_grammar_expansion() {
    let values = [1.0, 5.0, 2.0, 8.0];
    let frame = Frame::plain(40, 10);
    let preset = crate::line(&values[..]).render(&frame);
    let grammar = Plot::new().layer(Line::y(&values[..])).render(&frame);
    assert_eq!(preset, grammar);
}

#[test]
fn rendering_is_deterministic() {
    let plot = Plot::new().layer(Line::y(&[1.0, 5.0, 2.0, 8.0][..]));
    let frame = Frame::plain(40, 10);
    assert_eq!(plot.render(&frame), plot.render(&frame));
}

#[test]
fn no_frame_size_panics() {
    let plot = Plot::new()
        .layer(Line::y(&[1.0, f64::NAN, 2.0, 8.0][..]))
        .title("robustness");
    for width in 0..=42 {
        for height in 0..=8 {
            let _ = plot.render(&Frame::plain(width, height));
        }
    }
}

#[test]
fn empty_plots_render_bare_chrome() {
    let text = Plot::new().render(&Frame::plain(30, 8));
    assert!(text.contains('\u{2502}'), "missing y axis: {text}");
    assert!(text.contains('\u{2500}'), "missing x axis: {text}");
}

// Golden snapshots. Flush-left so the expected charts stay readable in this file;
// regenerate by rendering with the same frames and eyeballing the diff.

const SPIKY: &str = r"8 ┤                         ⡠⠊
  │                       ⡠⠊
  │       ⣀⠤⠒⠤⣀        ⢀⠔⠉
4 ┤    ⡠⠔⠊     ⠉⠒⠤⣀  ⢀⠔⠁
  │⢀⡠⠔⠉            ⠉⠒⠁
0 ┤⠁
  └┬────────┬───────┬────────┬
   0        1       2        3";

#[test]
fn a_small_line_chart_matches_its_snapshot() {
    let text = crate::line(&[1.0, 5.0, 2.0, 8.0][..]).render(&Frame::plain(30, 8));
    assert_eq!(text, SPIKY);
}

const GAPPY: &str = r"5 ┤                      ⢀⠤⠊
  │       ⢀⡠⠒⠁         ⡠⠔⠁
  │    ⢀⡠⠔⠁           ⠈
  │⣀⠤⠒⠊⠁
0 ┤
  └┬───────────┬───────────┬
  0.0         2.5        5.0";

#[test]
fn a_gap_breaks_the_line_in_the_snapshot() {
    let gappy = [1.0, 2.0, 4.0, f64::NAN, 3.0, 5.0];
    let text = crate::line(&gappy[..]).render(&Frame::plain(28, 7));
    assert_eq!(text, GAPPY);
}

const SINE: &str = r"           sin
1 ┤      ⢀⠤⠔⠚⠉⠉⠉⠓⠢⠤⡀
  │    ⡠⠜⠁         ⠈⠣⢄
  │  ⡤⠊               ⠑⢤
0 ┤⡠⠊                   ⠑⢄
  └┬────────────────────┬─
   0                    3";

#[test]
fn a_sampled_function_matches_its_snapshot() {
    let plot = Plot::new()
        .layer(Line::function(0.0..std::f64::consts::PI, f64::sin))
        .title("sin");
    assert_eq!(plot.render(&Frame::plain(26, 7)), SINE);
}
