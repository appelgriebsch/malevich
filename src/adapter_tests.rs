use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::widgets::Widget;

use crate::{Line, Plot};

#[test]
fn the_widget_draws_into_a_buffer_with_styles() {
    let plot = Plot::new()
        .layer(Line::y(&[1.0, 5.0, 2.0][..]).label("a"))
        .layer(Line::y(&[2.0, 1.0, 4.0][..]).label("b"))
        .title("w");
    let area = Rect::new(0, 0, 30, 10);
    let mut buffer = Buffer::empty(area);
    plot.widget().render(area, &mut buffer);

    let content: String = (0..10)
        .map(|y| {
            (0..30)
                .map(|x| buffer[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(content.contains('\u{2502}'), "missing axis: {content}");
    assert!(content.contains('w'), "missing title: {content}");
    // Palette colors arrived as styles, not escapes.
    let styled =
        (0..30).any(|x| (0..10).any(|y| buffer[(x, y)].fg != ratatui_core::style::Color::Reset));
    assert!(styled, "no colored cells");
}

#[test]
fn rendering_clips_to_the_area() {
    let plot = Plot::new().layer(Line::y(&[1.0, 2.0][..]));
    let area = Rect::new(2, 1, 10, 5);
    let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 10));
    plot.widget().render(area, &mut buffer);
    for x in 0..20u16 {
        assert_eq!(buffer[(x, 0)].symbol(), " ", "wrote outside the area");
    }
}
