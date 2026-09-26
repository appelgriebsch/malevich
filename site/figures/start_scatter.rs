{
    use super::penguins;
    let p = penguins();
    malevich::scatter(p.bill_length, p.bill_depth)
        .title("penguin bills")
        .x_label("length, mm")
        .y_label("depth, mm")
}
