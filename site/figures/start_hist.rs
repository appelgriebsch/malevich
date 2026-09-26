{
    use super::penguins;
    // Penguin body mass in grams: a real, lumpy distribution.
    let mass = penguins().mass;
    malevich::hist(mass).title("body mass, g").x_label("g")
}
