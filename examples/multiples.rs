//! Small multiples: a Grid pastes independently rendered plots side by side.
//! Shared axes are a composition — fix them with `y_domain` — not a mode.

use malevich::{Frame, Grid};

fn main() {
    let a: Vec<f64> = (0..50).map(|i| (i as f64 * 0.2).sin() * 3.0).collect();
    let b: Vec<f64> = (0..50).map(|i| (i as f64 * 0.13).cos() * 5.0).collect();
    let grid = Grid::new(2)
        .with(malevich::line(&a[..]).title("alpha").y_domain(-6.0, 6.0))
        .with(malevich::line(&b[..]).title("beta").y_domain(-6.0, 6.0))
        .with(malevich::hist(&a[..]).title("alpha dist"))
        .with(malevich::hist(&b[..]).title("beta dist"));
    println!("{}", grid.render(&Frame::plain(76, 22)));
}
