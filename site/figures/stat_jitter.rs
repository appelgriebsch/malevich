{
    use malevich::stat::jitter;
    use malevich::{Plot, Points, Scale};
    use super::penguins;
    // A strip plot: every measurement, spread across its band by van der Corput
    // offsets — even, seedless, the same every time.
    let p = penguins();
    let species = ["Adelie", "Chinstrap", "Gentoo"];
    let mut plot = Plot::new().x_scale(Scale::bands(species));
    for (index, name) in species.iter().enumerate() {
        let masses = p.of(name, &p.mass);
        let positions = jitter(&vec![index as f64; masses.len()], 0.7);
        plot = plot.layer(Points::xy(positions, masses));
    }
    plot.title("stat::jitter").y_label("g")
}
