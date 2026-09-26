{
    use malevich::stat::Fit;
    use malevich::{Dash, Line, Plot, Points, Rule, Text};
    use super::penguins;
    let p = penguins();
    let mut plot = Plot::new().layer(Points::xy(p.bill_length.clone(), p.bill_depth.clone()).color_by(p.species.clone()));
    for species in ["Adelie", "Chinstrap", "Gentoo"] {
        let (x, y) = (p.of(species, &p.bill_length), p.of(species, &p.bill_depth));
        let fit = Fit::xy(&x, &y);
        let (lo, hi) = (x.iter().cloned().fold(f64::MAX, f64::min), x.iter().cloned().fold(f64::MIN, f64::max));
        let xs = vec![lo, hi];
        let ys: Vec<f64> = xs.iter().map(|&x| fit.predict(x).unwrap_or(f64::NAN)).collect();
        plot = plot.layer(Line::xy(xs, ys));
    }
    let pooled = Fit::xy(&p.bill_length, &p.bill_depth);
    plot.layer(Rule::h(pooled.predict(44.0).unwrap_or(17.0)).dash(Dash::Dotted).label("pooled mean depth"))
        .layer(Text::at(33.0, 13.6, "pooled slope is negative"))
        .title("Simpson's paradox in penguin bills")
        .x_label("bill length, mm")
        .y_label("depth, mm")
}
