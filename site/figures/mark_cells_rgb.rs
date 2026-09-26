{
    use malevich::{Cells, Plot};
    // Direct colors, no colormap: an image is one more cell grid.
    let (w, h) = (48, 20);
    let mut pixels = Vec::with_capacity(w * h);
    for row in 0..h {
        for column in 0..w {
            let (x, y) = (column as f64 / w as f64, row as f64 / h as f64);
            let r = (255.0 * (0.5 + 0.5 * (x * 6.3).sin())) as u8;
            let g = (255.0 * (0.5 + 0.5 * (y * 6.3 + 1.0).sin())) as u8;
            let b = (255.0 * (0.5 + 0.5 * ((x + y) * 4.0).cos())) as u8;
            pixels.push((r, g, b));
        }
    }
    Plot::new().layer(Cells::rgb(w, pixels)).axes(false).title("Cells::rgb")
}
