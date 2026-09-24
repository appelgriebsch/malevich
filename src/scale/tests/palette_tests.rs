use super::Palette;
use crate::render::Color;

#[test]
fn the_default_palette_is_okabe_ito() {
    assert_eq!(Palette::default(), Palette::OKABE_ITO);
    assert_eq!(Palette::OKABE_ITO.colors().len(), 7);
    assert_eq!(Palette::OKABE_ITO.colors()[0], Color::Rgb(230, 159, 0));
}

#[test]
fn categories_past_the_palette_wrap_around() {
    let palette = Palette::new(&[Color::Red, Color::Green]);
    assert_eq!(palette.color(0), Color::Red);
    assert_eq!(palette.color(1), Color::Green);
    assert_eq!(palette.color(2), Color::Red);
    assert_eq!(palette.color(5), Color::Green);
}

#[test]
fn runtime_colors_move_into_an_owned_palette() {
    let colors = vec![Color::Cyan, Color::Rgb(1, 2, 3)];
    let palette = Palette::try_from_colors(colors).unwrap();
    assert_eq!(palette.colors(), [Color::Cyan, Color::Rgb(1, 2, 3)]);
}

#[test]
fn a_runtime_palette_requires_a_color() {
    assert!(matches!(
        Palette::try_from_colors(Vec::new()),
        Err(crate::Error::EmptyDimension {
            what: "Palette colors"
        })
    ));
}

#[test]
fn okabe_ito_survives_the_16_color_quantizer_distinguishably() {
    use crate::render::color::rgb_to_16;

    // In 16-color output adjacent categories must not collapse into one
    // another: each color's quantized index differs from its neighbor's.
    let quantized: Vec<u8> = Palette::OKABE_ITO
        .colors()
        .iter()
        .map(|color| match color {
            Color::Rgb(r, g, b) => rgb_to_16(*r, *g, *b),
            other => panic!("Okabe–Ito is concrete RGB, got {other:?}"),
        })
        .collect();
    for pair in quantized.windows(2) {
        assert_ne!(
            pair[0], pair[1],
            "neighbors collapse at 16 colors: {quantized:?}"
        );
    }
}

#[test]
fn tols_palettes_stand_beside_okabe_ito() {
    use crate::render::color::rgb_to_256;

    assert_eq!(Palette::BRIGHT.colors().len(), 7);
    assert_eq!(Palette::BRIGHT.colors()[0], Color::Rgb(68, 119, 170));
    assert_eq!(Palette::MUTED.colors().len(), 9);
    assert_eq!(Palette::MUTED.colors()[0], Color::Rgb(51, 34, 136));
    // Every color of each scheme survives the 256-color tier distinct.
    for palette in [Palette::BRIGHT, Palette::MUTED] {
        let mut codes: Vec<u8> = palette
            .colors()
            .iter()
            .map(|color| match color {
                Color::Rgb(r, g, b) => rgb_to_256(*r, *g, *b),
                other => panic!("Tol's colors are RGB, got {other:?}"),
            })
            .collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), palette.colors().len());
    }
}
