{
    use malevich::stat::Fit;
    use malevich::{Line, Plot, Points};
    use super::penguins;
    let p = penguins();
    // One streaming least-squares accumulator per species, one line each.
    let mut plot = Plot::new().layer(Points::xy(p.bill_length.clone(), p.bill_depth.clone()).color_by(p.species.clone()));
    for species in ["Adelie", "Chinstrap", "Gentoo"] {
        let (x, y) = (p.of(species, &p.bill_length), p.of(species, &p.bill_depth));
        let fit = Fit::xy(&x, &y);
        let (lo, hi) = (x.iter().cloned().fold(f64::MAX, f64::min), x.iter().cloned().fold(f64::MIN, f64::max));
        let xs = vec![lo, hi];
        let ys: Vec<f64> = xs.iter().map(|&x| fit.predict(x).unwrap_or(f64::NAN)).collect();
        plot = plot.layer(Line::xy(xs, ys));
    }
    plot
}
