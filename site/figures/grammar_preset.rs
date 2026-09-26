{
    use super::penguins;
    let p = penguins();
    // The same first step, through the front door.
    malevich::scatter(p.bill_length, p.bill_depth)
}
