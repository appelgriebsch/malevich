{
    use malevich::stat::{binned, Bins, Reducer};
    use malevich::{Bars, Line, Plot, Points};
    let mut unit = super::noise(19);
    // Confidence versus accuracy: 0/1 outcomes, binned by confidence, mean per bin.
    let confidence: Vec<f64> = (0..2000).map(|_| unit()).collect();
    let correct: Vec<f64> = confidence.iter().map(|c| if unit() < c * 0.8 + 0.1 { 1.0 } else { 0.0 }).collect();
    let bins = Bins::new(0.0, 0.1, 10);
    let accuracy = binned(&confidence, &correct, &bins, Reducer::Mean);
    let centers: Vec<f64> = (0..10).map(|i| 0.05 + 0.1 * i as f64).collect();
    Plot::new()
        .layer(Bars::spans(0.0, 0.1, accuracy.clone()).label("accuracy per bin"))
        .layer(Points::xy(centers, accuracy))
        .layer(Line::xy(vec![0.0, 1.0], vec![0.0, 1.0]).label("perfect calibration"))
        .title("stat::binned with Reducer::Mean")
        .x_label("confidence")
        .y_label("accuracy")
}
