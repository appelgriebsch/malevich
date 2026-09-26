{
    use malevich::stat::{kde_with, Bandwidth, KdeOptions};
    use malevich::{Line, Plot};
    use super::penguins;
    // The bandwidth is the one honest knob a density has: the same 342 masses
    // at Silverman's rule, at half of it, and at twice it.
    let mass = penguins().mass;
    let mut plot = Plot::new();
    for (scale, label) in [(0.5, "½ Silverman"), (1.0, "Silverman (default)"), (2.0, "2× Silverman")] {
        let options = KdeOptions::new().bandwidth(Bandwidth::Scale(scale));
        let (x, y) = kde_with(&mass, 200, options).expect("valid").expect("data");
        plot = plot.layer(Line::xy(x, y).label(label));
    }
    plot.title("kernel density, three bandwidths").x_label("body mass, g")
}
