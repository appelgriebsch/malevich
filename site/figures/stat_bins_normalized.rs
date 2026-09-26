{
    use malevich::stat::Normalization;
    use malevich::{hist_with, HistogramOptions};
    let mut unit = super::noise(3);
    // Latencies: log-normal, the long tail every service has.
    let latency: Vec<f64> = (0..4000).map(|_| (0.4 * super::gaussian(&mut unit) + 3.2).exp()).collect();
    hist_with(latency, HistogramOptions::new(40).normalization(Normalization::Percent).cumulative(true))
        .expect("valid options")
        .title("cumulative share of requests")
        .x_label("ms")
        .y_label("%")
}
