{
    use super::penguins;
    let p = penguins();
    let groups = ["Adelie", "Chinstrap", "Gentoo"].map(|s| p.of(s, &p.flipper));
    // Moments, quantiles, and extremes as a stat table: text on two band axes.
    malevich::describe(["Adelie", "Chinstrap", "Gentoo"], groups).title("flipper length, mm")
}
