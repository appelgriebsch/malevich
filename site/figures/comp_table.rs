{
    use malevich::scale::Colormap;
    use malevich::{table_with, TableOptions};
    // Any numeric matrix as an aligned table; a colormap positions each value
    // within its own column, so the digits survive any pipe and the color
    // shows where a terminal has one.
    let quarters = ["Q1", "Q2", "Q3", "Q4"];
    let metrics = ["revenue", "margin", "churn"];
    let values = vec![
        1.24e6, 0.31, 0.042,
        1.31e6, 0.33, 0.039,
        1.19e6, 0.28, 0.051,
        1.52e6, 0.36, 0.035,
    ];
    table_with(quarters, metrics, values, TableOptions::new().colormap(Colormap::VIRIDIS))
        .expect("rectangular")
        .title("table_with — colored per column")
}
