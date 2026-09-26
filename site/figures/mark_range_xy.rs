{
    use malevich::{Plot, Points, Range};
    // An interval per measurement: error bars are a Range around each point.
    let dose = vec![1.0, 2.0, 4.0, 8.0, 16.0, 32.0];
    let response = vec![0.12, 0.21, 0.38, 0.55, 0.71, 0.77];
    let error = vec![0.04, 0.05, 0.06, 0.05, 0.07, 0.09];
    let low: Vec<f64> = response.iter().zip(&error).map(|(r, e)| r - e).collect();
    let high: Vec<f64> = response.iter().zip(&error).map(|(r, e)| r + e).collect();
    Plot::new()
        .layer(Range::xy(dose.clone(), low, high))
        .layer(Points::xy(dose, response))
        .log_x()
        .title("Range::xy — an interval at each x")
        .x_label("dose")
        .y_label("response")
}
