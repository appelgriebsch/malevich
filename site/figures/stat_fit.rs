{
    use malevich::stat::Fit;
    use super::penguins;
    let p = penguins();
    let (x, y) = (p.of("Gentoo", &p.flipper), p.of("Gentoo", &p.mass));
    let fit = Fit::xy(&x, &y);
    // The preset draws the points, the line, and the band; the accumulator answers the numbers.
    malevich::trend(x, y)
        .title(format!("Gentoo mass by flipper — slope {:.1} g/mm, R² {:.2}", fit.slope().unwrap_or(0.0), fit.r_squared().unwrap_or(0.0)))
        .x_label("flipper, mm")
        .y_label("g")
}
