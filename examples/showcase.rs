//! A colored tour of every mark and preset, rendered for *your* terminal.
//!
//! Uses `Frame::detect()`: charts size themselves to the terminal width, use color
//! when the terminal has any, and degrade to plain text when piped. This example is
//! deliberately not part of the deterministic gallery — its output depends on where
//! you run it, which is the point. For the moving version of this tour, run
//! `cargo run --example live`.

use malevich::{Area, Frame, Grid, Line, LineStyle, Plot, Points, Rule, Text};

fn main() {
    let frame = Frame::detect();

    // Lines, legend, annotations: the training-loop story.
    let steps: Vec<f64> = (0..120).map(f64::from).collect();
    let train: Vec<f64> = steps
        .iter()
        .map(|s| 3.8 * (-0.035 * s).exp() + 0.32 + 0.05 * (s * 0.7).sin())
        .collect();
    let val: Vec<f64> = steps
        .iter()
        .map(|s| 4.0 * (-0.03 * s).exp() + 0.55 + 0.08 * (s * 0.35).cos())
        .collect();
    println!(
        "{}\n",
        Plot::new()
            .layer(Line::xy(&steps[..], &train[..]).label("train"))
            .layer(Line::xy(&steps[..], &val[..]).label("val"))
            .layer(Rule::h(0.5).label("target"))
            .layer(Text::at(60.0, 2.0, "< converging"))
            .title("loss with annotations (synthetic)")
            .x_label("step")
            .y_label("loss")
            .render(&frame)
    );

    // A calendar axis: unix seconds in, "Aug 2" out.
    let month_stamp = |year: i64, month: u64| -> f64 {
        let y = year - i64::from(month <= 2);
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = (y - era * 400) as u64;
        let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        ((era * 146_097 + doe as i64 - 719_468) * 86_400) as f64
    };
    let stamps: Vec<f64> = (0..36)
        .map(|i| month_stamp(2024 + i / 12, (1 + i % 12) as u64))
        .collect();
    let level: Vec<f64> = (0..36)
        .map(|i| 400.0 + i as f64 * 0.2 + ((i % 12) as f64 * 0.52).sin() * 3.0)
        .collect();
    println!(
        "{}\n",
        Plot::new()
            .layer(Line::xy(&stamps[..], &level[..]))
            .title("a monthly series on a calendar axis (synthetic)")
            .time_x()
            .render(&frame)
    );

    // A rolling mean over its noisy source.
    let raw: Vec<f64> = (0..120)
        .map(|i| 3.0 * (-0.03 * i as f64).exp() + 0.4 + ((i * 7) % 13) as f64 * 0.06)
        .collect();
    let smooth = malevich::stat::Window::new(9).mean(&raw);
    println!(
        "{}\n",
        Plot::new()
            .layer(Line::y(&raw[..]).label("raw"))
            .layer(Line::y(&smooth[..]).label("rolling mean"))
            .title("smoothing (synthetic)")
            .render(&frame)
    );

    // Ten million points, downsampled pixel-exactly on the way in.
    let n = 10_000_000;
    let wave: Vec<f64> = (0..n)
        .map(|i| {
            let i = i as f64;
            (i * 0.0002).sin() * (i * 0.000013).cos() * 8.0
        })
        .collect();
    println!(
        "{}\n",
        malevich::line(&wave[..])
            .title("10,000,000 points through M4")
            .render(&frame)
    );

    // Bars and a histogram.
    println!(
        "{}\n",
        malevich::bar(
            ["rust", "go", "python", "typescript", "zig"],
            &[68.0, 41.0, 55.0, 62.0, 12.0][..],
        )
        .title("admired languages, % (synthetic)")
        .render(&frame)
    );
    let samples: Vec<f64> = (0..4000)
        .map(|i| {
            let i = i as f64;
            ((i * 0.731).sin() + (i * 1.13).sin() + (i * 2.71).sin()) * 2.0 + 10.0
        })
        .collect();
    println!(
        "{}\n",
        malevich::hist(&samples[..])
            .title("histogram, automatic bins")
            .render(&frame)
    );

    // Stacked areas.
    let x: Vec<f64> = (0..80).map(f64::from).collect();
    let solar: Vec<f64> = x.iter().map(|v| 3.0 + (v * 0.2).sin() + v * 0.02).collect();
    let wind: Vec<f64> = x
        .iter()
        .map(|v| 2.0 + (v * 0.13).cos().abs() * 1.5)
        .collect();
    let hydro: Vec<f64> = x.iter().map(|v| 1.0 + (v * 0.07).sin().abs()).collect();
    let bands = malevich::stat::stack(&[&solar, &wind, &hydro]);
    let mut stacked = Plot::new().title("energy mix, stacked (synthetic)");
    for ((low, high), label) in bands.iter().zip(["solar", "wind", "hydro"]) {
        stacked = stacked.layer(Area::between(&x[..], &low[..], &high[..]).label(label));
    }
    println!("{}\n", stacked.render(&frame));

    // A heatmap and a 2D histogram.
    let size = 8usize;
    let grid: Vec<f64> = (0..size * size)
        .map(|i| {
            let (row, column) = ((i / size) as f64, (i % size) as f64);
            if row == column {
                1.0
            } else {
                ((row - column).abs() * -0.4).exp()
            }
        })
        .collect();
    println!(
        "{}\n",
        malevich::heatmap(size, &grid[..])
            .title("correlation matrix (synthetic)")
            .render(&frame)
    );
    let bell = |i: f64, seed: f64| -> f64 {
        ((i * 0.97 + seed).sin() + (i * 1.31 + seed * 2.0).sin() + (i * 2.63 + seed * 3.0).sin())
            / 3.0
    };
    let points = 6000;
    let cx: Vec<f64> = (0..points)
        .map(|i| {
            let i = i as f64;
            if i as i64 % 2 == 0 {
                3.0 + bell(i, 1.0) * 1.8
            } else {
                7.0 + bell(i, 4.0) * 1.2
            }
        })
        .collect();
    let cy: Vec<f64> = (0..points)
        .map(|i| {
            let i = i as f64;
            if i as i64 % 2 == 0 {
                3.0 + bell(i, 7.0) * 1.4
            } else {
                6.5 + bell(i, 9.0) * 1.7
            }
        })
        .collect();
    println!(
        "{}\n",
        malevich::hist2d(&cx[..], &cy[..])
            .title("2d density (synthetic)")
            .render(&frame)
    );

    // Contour lines: marching squares over a saddle between two humps.
    let (columns, rows) = (40, 30);
    let mut z = Vec::with_capacity(columns * rows);
    for r in 0..rows {
        for c in 0..columns {
            let x = c as f64 / (columns - 1) as f64 * 4.0 - 2.0;
            let y = r as f64 / (rows - 1) as f64 * 4.0 - 2.0;
            z.push(
                (-(x - 0.8).powi(2) - (y - 0.6).powi(2)).exp()
                    - 0.8 * (-(x + 0.8).powi(2) - (y + 0.6).powi(2)).exp(),
            );
        }
    }
    println!(
        "{}\n",
        malevich::contour(columns, &z[..])
            .title("contour lines (synthetic)")
            .render(&frame)
    );

    // A vector field: circular flow, one arrow per grid point.
    let mut fx = Vec::new();
    let mut fy = Vec::new();
    let mut fu = Vec::new();
    let mut fv = Vec::new();
    for row in 0..8 {
        for column in 0..11 {
            let px = -2.0 + 0.4 * column as f64;
            let py = -1.4 + 0.4 * row as f64;
            fx.push(px);
            fy.push(py);
            fu.push(-0.3 * py);
            fv.push(0.3 * px);
        }
    }
    println!(
        "{}\n",
        malevich::quiver(&fx[..], &fy[..], &fu[..], &fv[..])
            .title("vector field (synthetic)")
            .render(&frame)
    );

    // The asciichart-style corners line.
    let wave: Vec<f64> = (0..60)
        .map(|i| 15.0 * (i as f64 * std::f64::consts::PI / 30.0).sin())
        .collect();
    println!(
        "{}\n",
        Plot::new()
            .layer(Line::y(&wave[..]).style(LineStyle::Corners))
            .title("the corners style")
            .render(&frame)
    );

    // Small multiples.
    let alpha: Vec<f64> = (0..50).map(|i| (i as f64 * 0.2).sin() * 3.0).collect();
    let beta: Vec<f64> = (0..50).map(|i| (i as f64 * 0.13).cos() * 5.0).collect();
    println!(
        "{}\n",
        Grid::new(2)
            .with(
                malevich::line(&alpha[..])
                    .title("alpha")
                    .y_domain(-6.0, 6.0)
            )
            .with(malevich::line(&beta[..]).title("beta").y_domain(-6.0, 6.0))
            .render(&frame)
    );

    // Log-log axes, an ECDF, and a labeled scatter to close.
    println!(
        "{}\n",
        Plot::new()
            .layer(Line::function(1.0..100_000.0, |x| 0.5 * x.powf(1.5)).label("0.5 x^1.5"))
            .layer(Line::function(1.0..100_000.0, |x| 20.0 * x.sqrt()).label("20 sqrt x"))
            .title("power laws, log-log")
            .log_x()
            .log_y()
            .render(&frame)
    );
    println!(
        "{}\n",
        malevich::ecdf(&samples[..])
            .title("ecdf of the histogram sample")
            .render(&frame)
    );
    let blob = |count: usize, cx: f64, cy: f64, spread: f64| -> (Vec<f64>, Vec<f64>) {
        (0..count)
            .map(|i| {
                let i = i as f64;
                (
                    cx + spread * (i * 0.97).sin() * (i * 0.31).cos(),
                    cy + spread * 0.6 * (i * 1.13).cos() * (i * 0.47).sin(),
                )
            })
            .unzip()
    };
    let (ax, ay) = blob(80, 3.0, 4.0, 1.6);
    let (bx, by) = blob(80, 7.5, 7.0, 1.9);
    println!(
        "{}",
        Plot::new()
            .layer(Points::xy(&ax[..], &ay[..]).label("colony a"))
            .layer(Points::xy(&bx[..], &by[..]).label("colony b"))
            .title("two colonies (synthetic)")
            .render(&frame)
    );
}
