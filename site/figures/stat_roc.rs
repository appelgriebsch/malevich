{
    use malevich::stat::{auc, roc};
    use malevich::{Dash, Line, Plot};
    let mut unit = super::noise(17);
    // Scores for 400 examples: positives score higher on average, with overlap.
    let labels: Vec<bool> = (0..400).map(|i| i % 3 == 0).collect();
    let scores: Vec<f64> = labels.iter().map(|&positive| super::gaussian(&mut unit) + if positive { 1.4 } else { 0.0 }).collect();
    let (fpr, tpr) = roc(&scores, &labels);
    let area = auc(&fpr, &tpr);
    Plot::new()
        .layer(Line::xy(fpr, tpr).label("classifier"))
        .layer(Line::xy(vec![0.0, 1.0], vec![0.0, 1.0]).dash(Dash::Dotted).label("chance"))
        .title(format!("ROC, AUC = {area:.3}"))
        .x_label("false positive rate")
        .y_label("true positive rate")
}
