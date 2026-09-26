{
    use malevich::stat::{steps, StepDirection};
    use malevich::{Line, Plot, Points};
    let x: Vec<f64> = (0..8).map(f64::from).collect();
    let y = vec![1.0, 3.0, 2.0, 5.0, 4.0, 6.0, 3.0, 4.0];
    let (ax, ay) = steps(&x, &y, StepDirection::Post);
    let (bx, by) = steps(&x, &y, StepDirection::Pre);
    let (mx, my) = steps(&x, &y, StepDirection::Mid);
    Plot::new()
        .layer(Points::xy(x, y).label("samples"))
        .layer(Line::xy(ax, ay).label("Post"))
        .layer(Line::xy(bx, by.iter().map(|y| y + 6.0).collect::<Vec<_>>()).label("Pre (+6)"))
        .layer(Line::xy(mx, my.iter().map(|y| y + 12.0).collect::<Vec<_>>()).label("Mid (+12)"))
        .title("stat::steps — where a value changes")
}
