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
fn the_scatter_preset_equals_its_grammar_expansion() {
    let x = [1.0, 2.0, 3.0];
    let y = [2.0, 1.0, 3.0];
    let frame = Frame::plain(40, 10);
    let preset = crate::scatter(&x[..], &y[..]).render(&frame);
    let grammar = Plot::new()
        .layer(crate::mark::Points::xy(&x[..], &y[..]))
        .render(&frame);
    assert_eq!(preset, grammar);
}

const PARABOLA: &str = r"10 ┤⠁                          ⠈
   │
   │  ⠁                      ⠈
 5 ┤    ⠄                  ⠠
   │      ⢀              ⡀
   │        ⢀          ⡀
 0 ┤          ⠠ ⢀  ⡀ ⠄
   └┬───────────┬────────────┬──
    0           3            6";

#[test]
fn scatter_dots_stay_unconnected_in_the_snapshot() {
    let x: Vec<f64> = (0..14).map(|i| i as f64 * 0.5).collect();
    let y: Vec<f64> = x.iter().map(|v| (v - 3.25) * (v - 3.25)).collect();
    let text = crate::scatter(&x[..], &y[..]).render(&Frame::plain(32, 9));
    assert_eq!(text, PARABOLA);
}

#[test]
fn the_bar_preset_equals_its_grammar_expansion() {
    let frame = Frame::plain(40, 10);
    let preset = crate::bar(["a", "b"], &[1.0, 2.0][..]).render(&frame);
    let grammar = Plot::new()
        .layer(crate::mark::Bars::new(["a", "b"], &[1.0, 2.0][..]))
        .render(&frame);
    assert_eq!(preset, grammar);
}

#[test]
fn large_lines_downsample_without_changing_the_picture() {
    let n = 50_000;
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..n)
        .map(|i| (i as f64 * 0.002).sin() * (i as f64 * 0.0003).cos() * 5.0)
        .collect();
    let frame = Frame::plain(70, 15);
    let full = Plot::new().layer(Line::xy(&x[..], &y[..])).render(&frame);
    // Manually downsampled to the same raster width; small enough to skip the
    // automatic path, so equality proves the auto-inserted M4 is pixel-exact.
    let (dx, dy) = crate::stat::m4(&x, &y, frame.width * 2).unwrap();
    let manual = Plot::new().layer(Line::xy(&dx[..], &dy[..])).render(&frame);
    assert_eq!(full, manual);
}

#[test]
fn labeled_layers_grow_a_legend_row() {
    let plot = Plot::new()
        .layer(Line::y(&[1.0, 2.0][..]).label("first"))
        .layer(Line::y(&[2.0, 1.0][..]).label("second"));
    let text = plot.render(&Frame::plain(40, 10));
    assert!(
        text.contains("\u{2500}\u{2500} first  \u{2500}\u{2500} second"),
        "missing legend: {text}"
    );
    // Shed before anything else when the frame is short.
    let short = plot.render(&Frame::plain(40, 7));
    assert!(!short.contains("first"), "legend not shed: {short}");
}

const BARS_WITH_TREND: &str = r"           bars with a trend line
7.5 ┤                             ▃▃▃▃▃▃▃
    │                             ███████
    │           ▁▁▁▁▁▁▁        ⢀⡠⠔███████
5.0 ┤           ███████⠒⠢⠤⠤⠤⠤⠔⠊⠁  ███████
    │         ⣀⠔███████  ▇▇▇▇▇▇▇  ███████
2.5 ┤  ▆▆▆▆▆▆▆  ███████  ███████  ███████
    │  ███████  ███████  ███████  ███████
    │  ███████  ███████  ███████  ███████
0.0 ┤  ███████  ███████  ███████  ███████
    └───────────────────────────────────────
          q1       q2       q3       q4";

#[test]
fn bars_share_scales_with_a_line_overlay_in_the_snapshot() {
    let text = Plot::new()
        .layer(crate::mark::Bars::new(
            ["q1", "q2", "q3", "q4"],
            &[3.0, 5.0, 4.0, 7.0][..],
        ))
        .layer(Line::y(&[2.5, 4.8, 4.4, 6.5][..]))
        .title("bars with a trend line")
        .render(&Frame::plain(44, 12));
    assert_eq!(text, BARS_WITH_TREND);
}

const NEGATIVE_BARS: &str = r" 7.5 ┤            ▄▄▄▄
     │            ████            ▄▄▄▄
 5.0 ┤            ████ ▁▁▁▁       ████      ▅▅▅▅
     │            ████ ████       ████      ████
     │ ▅▅▅▅       ████ ████       ████      ████
 2.5 ┤ ████       ████ ████       ████ ▇▇▇▇ ████
     │ ████       ████ ████       ████ ████ ████
 0.0 ┤ ████  ████ ████ ████ ████  ████ ████ ████
     │       ████           ▔▔▔▔
-2.5 ┤       ▔▔▔▔
     └────────────────────────────────────────────
         a     b    c    d    e     f    g    h";

#[test]
fn negative_bars_hang_below_the_baseline_in_the_snapshot() {
    let text = crate::bar(
        ["a", "b", "c", "d", "e", "f", "g", "h"],
        &[3.0, -2.0, 7.0, 4.5, -1.2, 6.0, 2.2, 5.0][..],
    )
    .render(&Frame::plain(50, 12));
    assert_eq!(text, NEGATIVE_BARS);
}

#[test]
fn log_axes_straighten_exponentials_and_drop_nonpositives() {
    let steps: Vec<f64> = (0..40).map(f64::from).collect();
    let decay: Vec<f64> = steps.iter().map(|s| 100.0 * (-0.3 * s).exp()).collect();
    let text = Plot::new()
        .layer(Line::xy(&steps[..], &decay[..]))
        .log_y()
        .render(&Frame::plain(40, 10));
    assert!(
        text.contains("10\u{2077}") || text.contains("10\u{207B}"),
        "no decade labels: {text}"
    );

    let with_zeroes = Plot::new()
        .layer(Line::y(&[1.0, 0.0, -5.0, 100.0][..]))
        .log_y()
        .render(&Frame::plain(40, 10));
    assert!(!with_zeroes.is_empty());
}

#[test]
fn the_hist_preset_equals_its_grammar_expansion() {
    let samples: Vec<f64> = (0..500).map(|i| ((i * 37) % 100) as f64 / 10.0).collect();
    let frame = Frame::plain(50, 12);
    let preset = crate::hist(&samples[..]).render(&frame);
    let bins = crate::stat::Bins::auto(&samples, 60).unwrap();
    let counts: Vec<f64> = bins.counts().iter().map(|&c| c as f64).collect();
    let grammar = Plot::new()
        .layer(crate::mark::Bars::spans(
            bins.start(),
            bins.width(),
            &counts[..],
        ))
        .render(&frame);
    assert_eq!(preset, grammar);
}

#[test]
fn span_bars_sit_contiguously_on_a_numeric_axis() {
    let text = Plot::new()
        .layer(crate::mark::Bars::spans(0.0, 1.0, &[2.0, 5.0, 3.0][..]))
        .render(&Frame::plain(40, 10));
    // A numeric axis (ticks, not category labels) under contiguous bars.
    assert!(text.contains('\u{252C}'), "missing numeric ticks: {text}");
    assert!(text.contains('\u{2588}'), "missing bar fills: {text}");
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
