//! Function sampling: curves drawn from `f(x)`, one sample per subpixel column.

use malevich::{Frame, Line, Plot};

fn main() {
    let plot = Plot::new()
        .layer(Line::function(0.0..12.6, f64::sin))
        .layer(Line::function(0.0..12.6, |x| (x * 0.5).cos() * 0.6))
        .title("sin(x) and 0.6 cos(x/2)");
    println!("{}", plot.render_best(&Frame::plain(72, 16)));
}
