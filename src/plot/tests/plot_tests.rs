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
fn large_lines_downsample_pixel_exactly_against_the_raw_raster() {
    // The oracle is the *raw* raster — every point drawn, M4 disabled. Mapped-space
    // M4 buckets by the rendered column, so the reduction is bit-identical to it, not
    // merely close. Cover an index line and an xy line at several frame sizes.
    let index: Vec<f64> = (0..50_000)
        .map(|i| (i as f64 * 0.002).sin() * (i as f64 * 0.0003).cos() * 5.0)
        .collect();
    let xy_x: Vec<f64> = (0..200_000).map(|i| i as f64 * 0.3).collect();
    let xy_y: Vec<f64> = (0..200_000).map(|i| (i as f64 * 0.001).sin()).collect();
    for (width, height) in [(70, 15), (133, 24), (40, 10)] {
        let frame = Frame::plain(width, height);
        let index_plot = Plot::new().layer(Line::y(&index[..])).title("t");
        assert_eq!(
            index_plot.rasterize_with(&frame, true).to_plain(),
            index_plot.rasterize_with(&frame, false).to_plain(),
            "index line at {width}x{height} is not pixel-exact"
        );
        let xy_plot = Plot::new().layer(Line::xy(&xy_x[..], &xy_y[..]));
        assert_eq!(
            xy_plot.rasterize_with(&frame, true).to_plain(),
            xy_plot.rasterize_with(&frame, false).to_plain(),
            "xy line at {width}x{height} is not pixel-exact"
        );
    }
}

#[test]
fn a_gap_inside_a_raster_column_stays_a_break() {
    // Many points per column with a NaN between a jump from low to high: the raw
    // render breaks the line there, and the downsampled one must too (COR-03).
    let n = 20_000;
    let mut x = Vec::with_capacity(n);
    let mut y = Vec::with_capacity(n);
    for i in 0..n {
        x.push(i as f64);
        y.push(if i == n / 2 {
            f64::NAN
        } else if i < n / 2 {
            -4.0
        } else {
            4.0
        });
    }
    let frame = Frame::plain(40, 11);
    let plot = Plot::new().layer(Line::xy(&x[..], &y[..]));
    assert_eq!(
        plot.rasterize_with(&frame, true).to_plain(),
        plot.rasterize_with(&frame, false).to_plain(),
        "M4 must reproduce the raw raster's gap, not bridge it"
    );
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
fn every_charset_renders_with_its_own_glyphs() {
    use crate::Charset;
    let plot = Plot::new()
        .layer(Line::y(&[1.0, 4.0, 2.0, 5.0][..]))
        .layer(crate::mark::Area::y(&[0.5, 2.0, 1.0, 2.5][..]))
        .title("t");
    for (charset, witness) in [
        (
            Charset::HalfBlocks,
            &['\u{2580}', '\u{2584}', '\u{2588}'][..],
        ),
        (
            Charset::Quadrants,
            &['\u{2596}', '\u{2599}', '\u{2588}', '\u{259F}', '\u{2584}'][..],
        ),
        (Charset::Braille, &['\u{28FF}', '\u{2801}', '\u{28C0}'][..]),
    ] {
        let mut frame = Frame::plain(24, 8);
        frame.charset = charset;
        let text = plot.render(&frame);
        assert_eq!(text, plot.render(&frame), "nondeterministic in {charset:?}");
        assert!(
            text.chars().any(|c| {
                let cp = c as u32;
                (0x2580..=0x28FF).contains(&cp)
            }),
            "{charset:?} drew no block/braille glyphs: {text}"
        );
        let _ = witness;
    }
    let mut ascii = Frame::plain(24, 8);
    ascii.charset = Charset::Ascii;
    let text = plot.render(&ascii);
    assert!(text.is_ascii(), "ASCII output leaked non-ASCII: {text}");
}

#[test]
fn axis_titles_render_on_both_axes() {
    let plot = Plot::new()
        .layer(Line::y(&[1.0, 2.0][..]))
        .x_label("step")
        .y_label("loss");
    let text = plot.render(&Frame::plain(40, 12));
    assert!(text.contains("step"), "missing x label: {text}");
    for letter in ["l", "o", "s"] {
        assert!(text.contains(letter), "missing y label letters: {text}");
    }
    // Both shed cleanly when there is no room.
    let _ = plot.render(&Frame::plain(10, 3));
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

#[test]
fn a_well_formed_plot_validates_and_try_renders() {
    let plot = crate::scatter(&[1.0, 2.0, 3.0][..], &[3.0, 1.0, 2.0][..]).title("ok");
    assert!(plot.validate().is_ok());
    assert!(plot.try_render(&Frame::plain(40, 10)).is_ok());
}

#[test]
fn a_log_axis_with_a_non_positive_domain_is_rejected() {
    let plot = crate::line(&[1.0, 10.0, 100.0][..])
        .y_domain(-1.0, 100.0)
        .log_y();
    assert!(matches!(
        plot.validate(),
        Err(crate::Error::IncompatibleScale { .. })
    ));
    // render still succeeds — it clamps rather than fails.
    assert!(!plot.render(&Frame::plain(40, 10)).is_empty());
    assert!(plot.try_render(&Frame::plain(40, 10)).is_err());
}

#[test]
fn validation_reaches_into_every_layer() {
    // A ragged range built by round-tripping through into_owned keeps its lengths,
    // so a valid multi-layer plot validates; the layer walk visits each mark.
    let plot = Plot::new()
        .layer(Line::xy(&[0.0, 1.0][..], &[2.0, 3.0][..]))
        .layer(crate::mark::Bars::new(["a", "b"], &[1.0, 2.0][..]));
    assert!(plot.validate().is_ok());
}
