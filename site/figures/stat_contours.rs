{
    // The MATLAB peaks function as iso-lines: marching squares, tick-chosen levels.
    let n = 60;
    let mut values = Vec::with_capacity(n * n);
    for row in 0..n {
        for column in 0..n {
            let (x, y) = (column as f64 / n as f64 * 6.0 - 3.0, 3.0 - row as f64 / n as f64 * 6.0);
            values.push(3.0 * (1.0 - x).powi(2) * (-x * x - (y + 1.0).powi(2)).exp() - 10.0 * (x / 5.0 - x.powi(3) - y.powi(5)) * (-x * x - y * y).exp() - (-(x + 1.0).powi(2) - y * y).exp() / 3.0);
        }
    }
    malevich::contour(n, values).title("contour — peaks")
}
