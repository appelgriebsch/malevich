{
    use malevich::stat::Whiskers;
    use malevich::{box_plot_with, BoxOptions};
    use super::penguins;
    let p = penguins();
    let groups = ["Adelie", "Chinstrap", "Gentoo"].map(|s| p.of(s, &p.mass));
    // Whiskers to the 5th and 95th percentiles instead of Tukey's 1.5 IQR.
    box_plot_with(["Adelie", "Chinstrap", "Gentoo"], groups, BoxOptions::new().whiskers(Whiskers::Percentiles(0.05, 0.95)))
        .expect("valid")
        .title("body mass, whiskers at p5 and p95")
        .y_label("g")
}
