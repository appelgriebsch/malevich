{
    use malevich::stat::BoxStats;
    use malevich::{Plot, Range};
    use super::penguins;
    // The box plot, spelled out: whiskers, a body from q1 to q3, a marker at the median.
    let p = penguins();
    let species = ["Adelie", "Chinstrap", "Gentoo"];
    let stats: Vec<BoxStats> = species.iter().map(|s| BoxStats::of(&p.of(s, &p.flipper)).expect("data")).collect();
    Plot::new()
        .layer(
            Range::over(species, stats.iter().map(|s| s.whisker_low).collect::<Vec<_>>(), stats.iter().map(|s| s.whisker_high).collect::<Vec<_>>())
                .body(stats.iter().map(|s| s.q1).collect::<Vec<_>>(), stats.iter().map(|s| s.q3).collect::<Vec<_>>())
                .marker(stats.iter().map(|s| s.median).collect::<Vec<_>>()),
        )
        .title("Range::over with body and marker")
        .y_label("flipper, mm")
}
