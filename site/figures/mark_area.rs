{
    use malevich::{Area, Line, Plot};
    let x: Vec<f64> = (0..60).map(f64::from).collect();
    let mid: Vec<f64> = x.iter().map(|x| 10.0 + 4.0 * (x * 0.15).sin()).collect();
    let low: Vec<f64> = mid.iter().zip(&x).map(|(m, x)| m - 1.0 - x * 0.03).collect();
    let high: Vec<f64> = mid.iter().zip(&x).map(|(m, x)| m + 1.0 + x * 0.03).collect();
    let floor: Vec<f64> = x.iter().map(|x| 2.0 + (x * 0.3).cos()).collect();
    Plot::new()
        .layer(Area::xy(x.clone(), floor).label("Area::xy, from the baseline"))
        .layer(Area::between(x.clone(), low, high).opacity(0.35).label("Area::between, a band"))
        .layer(Line::xy(x, mid).label("the line inside it"))
        .title("areas")
}
