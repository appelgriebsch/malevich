{
    use malevich::{Cells, Plot, Points};
    // Categorical regions: each cell names a class; the legend follows the palette.
    let n = 40;
    let mut labels = Vec::with_capacity(n * n);
    for row in 0..n {
        for column in 0..n {
            let (x, y) = (column as f64 / n as f64 * 4.0 - 2.0, 2.0 - row as f64 / n as f64 * 4.0);
            labels.push(if (x * x + y * y).sqrt() < 1.0 { "inside" } else if x + y > 1.4 { "north-east" } else { "outside" });
        }
    }
    let ring: Vec<f64> = (0..24).map(|i| f64::from(i) * 0.26).collect();
    Plot::new()
        .layer(Cells::classes(n, labels).extents((-2.0, 2.0), (-2.0, 2.0)))
        .layer(Points::xy(ring.iter().map(|t| t.cos() * 1.05).collect::<Vec<_>>(), ring.iter().map(|t| t.sin() * 1.05).collect::<Vec<_>>()).label("boundary points"))
        .title("Cells::classes — a decision region")
}
