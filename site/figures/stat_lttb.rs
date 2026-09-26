{
    use malevich::stat::lttb;
    use malevich::{Line, Plot, Points};
    let x: Vec<f64> = (0..3000).map(|i| f64::from(i) * 0.01).collect();
    let y: Vec<f64> = x.iter().map(|x| (x * 2.0).sin() * (x * 0.3).cos() * 3.0 + (x * 17.0).sin() * 0.3).collect();
    // LTTB keeps 120 visually representative points. It is explicit and inexact —
    // a stat the caller applies — never a default reduction.
    let (kept_x, kept_y) = lttb(&x, &y, 120);
    Plot::new()
        .layer(Line::xy(x, y).label("3,000 points"))
        .layer(Points::xy(kept_x, kept_y).label("120 kept by LTTB"))
        .title("stat::lttb — an opt-in approximation")
}
