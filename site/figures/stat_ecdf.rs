{
    use malevich::{ecdf_with, EcdfOptions};
    let mut unit = super::noise(13);
    let latency: Vec<f64> = (0..600).map(|_| (0.35 * super::gaussian(&mut unit) + 3.0).exp()).collect();
    // The empirical CDF, with the Dvoretzky–Kiefer–Wolfowitz 95 % band.
    ecdf_with(latency, EcdfOptions::new().band(0.05))
        .expect("valid")
        .title("ECDF with a DKW band")
        .x_label("ms")
}
