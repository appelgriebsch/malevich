//! One compact Suprematist plot, rendered side by side as octants and real pixels.
//!
//! ```sh
//! cargo run --example suprematist --features pixel
//! ```

use malevich::{Area, Bars, Charset, Color, Frame, Line, Plot, Points, Range};

const INK: Color = Color::Rgb(31, 31, 29);
const AUBERGINE: Color = Color::Rgb(54, 34, 45);
const BLUE: Color = Color::Rgb(7, 54, 151);
const GREEN: Color = Color::Rgb(0, 155, 84);
const RED: Color = Color::Rgb(218, 55, 27);
const ORANGE: Color = Color::Rgb(238, 88, 27);
const GOLD: Color = Color::Rgb(247, 153, 8);
const YELLOW: Color = Color::Rgb(242, 194, 0);
const PINK: Color = Color::Rgb(225, 139, 174);

#[derive(Clone, Copy)]
struct Block {
    center: (f64, f64),
    size: (f64, f64),
    angle: f64,
    color: Color,
}

impl Block {
    const fn new(center: (f64, f64), size: (f64, f64), angle: f64, color: Color) -> Block {
        Block {
            center,
            size,
            angle,
            color,
        }
    }

    /// Turns a rotated rectangle into the lower and upper edges of an `Area`.
    fn mark(self) -> Area<'static> {
        let angle = self.angle.to_radians();
        let (sin, cos) = angle.sin_cos();
        let (half_width, half_height) = (self.size.0 / 2.0, self.size.1 / 2.0);
        let corners = [
            (-half_width, -half_height),
            (half_width, -half_height),
            (half_width, half_height),
            (-half_width, half_height),
        ]
        .map(|(x, y)| {
            (
                self.center.0 + x * cos - y * sin,
                self.center.1 + x * sin + y * cos,
            )
        });

        let left = corners
            .iter()
            .map(|corner| corner.0)
            .fold(f64::INFINITY, f64::min);
        let right = corners
            .iter()
            .map(|corner| corner.0)
            .fold(f64::NEG_INFINITY, f64::max);
        let mut x = Vec::with_capacity(65);
        let mut low = Vec::with_capacity(65);
        let mut high = Vec::with_capacity(65);

        // A rectangle's silhouette is piecewise linear; these samples are denser
        // than the terminal raster and remain smooth when pixel output is active.
        for sample in 0..=64 {
            let px = left + (right - left) * f64::from(sample) / 64.0;
            let mut intersections = Vec::with_capacity(8);
            for edge in 0..4 {
                let from = corners[edge];
                let to = corners[(edge + 1) % 4];
                let dx = to.0 - from.0;
                if dx.abs() < f64::EPSILON {
                    if (px - from.0).abs() < 1e-9 {
                        intersections.extend([from.1, to.1]);
                    }
                    continue;
                }
                let t = (px - from.0) / dx;
                if (-1e-9..=1.0 + 1e-9).contains(&t) {
                    intersections.push(from.1 + t * (to.1 - from.1));
                }
            }
            if let (Some(bottom), Some(top)) = (
                intersections.iter().copied().reduce(f64::min),
                intersections.iter().copied().reduce(f64::max),
            ) {
                x.push(px);
                low.push(bottom);
                high.push(top);
            }
        }
        Area::between(x, low, high).color(self.color)
    }
}

fn composition() -> Plot<'static> {
    // Back-to-front order makes overlap part of the composition.
    let blocks = [
        Block::new((28.0, 94.0), (43.0, 10.0), 66.0, GREEN),
        Block::new((17.0, 113.0), (21.0, 5.0), 67.0, INK),
        Block::new((12.0, 88.0), (18.0, 4.0), 63.0, INK),
        Block::new((61.0, 71.0), (39.0, 27.0), -32.0, BLUE),
        Block::new((14.0, 29.0), (22.0, 6.0), 62.0, ORANGE),
        Block::new((48.0, 14.0), (26.0, 7.0), 62.0, INK),
        Block::new((86.0, 19.0), (15.0, 6.0), 40.0, PINK),
    ];

    let plot = Plot::new()
        .title("suprematist composition")
        .x_domain(0.0, 100.0)
        .y_domain(0.0, 125.0)
        // A long field, a diagonal, and a small histogram establish the main axes.
        .layer(Block::new((51.0, 50.0), (72.0, 5.0), 0.0, AUBERGINE).mark())
        .layer(Line::xy([32.0, 76.0], [45.0, 116.0]).color(RED))
        .layer(Bars::spans(43.0, 5.5, [8.0, 20.0, 27.0, 13.0]).color(GOLD));

    let plot = blocks
        .into_iter()
        .fold(plot, |plot, block| plot.layer(block.mark()));

    plot
        // Box-plot geometry becomes the yellow burst in the upper-right.
        .layer(
            Range::xy([79.0, 93.0], [87.0, 91.0], [113.0, 108.0])
                .body([95.0, 97.0], [107.0, 104.0])
                .marker([101.0, 101.0])
                .color(YELLOW),
        )
        // A scatter constellation keeps the small upper forms airy.
        .layer(Points::xy([60.0, 66.0, 72.0], [111.0, 121.0, 112.0]).color(BLUE))
}

#[cfg(feature = "pixel")]
fn comparison(plot: &Plot<'_>, frame: &Frame) -> String {
    use std::fmt::Write as _;

    let pane = Frame {
        width: frame.width.saturating_sub(2) / 2,
        charset: Charset::Octants,
        ..*frame
    };
    let Some(graphics) = malevich::pixel::Graphics::detect() else {
        return plot
            .clone()
            .title("OCTANTS (pixel unavailable)")
            .render(&pane);
    };

    let mut output = plot.clone().title("OCTANTS").render(&pane);
    if pane.height > 1 {
        let _ = write!(output, "\x1b[{}A", pane.height - 1);
    }
    output.push_str(&plot.clone().title("PIXELS").render_pixels_at(
        &pane,
        &graphics,
        pane.width + 2,
    ));
    output
}

#[cfg(not(feature = "pixel"))]
fn comparison(plot: &Plot<'_>, frame: &Frame) -> String {
    plot.clone()
        .title("OCTANTS (build with pixel)")
        .render(&Frame {
            width: frame.width.saturating_sub(2) / 2,
            charset: Charset::Octants,
            ..*frame
        })
}

fn main() {
    let plot = composition();
    let detected = Frame::detect();
    let frame = Frame {
        width: 80,
        height: 24,
        ..detected
    };
    println!("{}", comparison(&plot, &frame));
}
