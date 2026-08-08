//! The ratatui adapter: a plot as a widget (feature `ratatui`).
//!
//! Depends only on `ratatui-core` — the stable trait-and-buffer layer — so any app
//! in the ratatui ecosystem can embed charts without version lockstep. The plot
//! rasterizes straight into the `Buffer`: no ANSI round-trip, colors map one to one
//! onto ratatui styles, and the terminal stays entirely the host application's.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Color as RatColor;
use ratatui_core::widgets::Widget;

use crate::plot::{Frame, Plot};
use crate::render::{Charset, Color, ColorMode};
use crate::theme::Theme;

impl Plot<'_> {
    /// Wraps the plot as a ratatui widget rendering into its area.
    ///
    /// ```no_run
    /// # fn draw(frame: &mut ratatui_core::terminal::Frame, area: ratatui_core::layout::Rect) {
    /// let chart = malevich::line(&[1.0, 4.0, 2.0][..]).title("demo");
    /// frame.render_widget(chart.widget(), area);
    /// # }
    /// ```
    pub fn widget(&self) -> PlotWidget<'_> {
        PlotWidget {
            plot: self,
            charset: Charset::Quadrants,
            theme: Theme::DARK,
        }
    }
}

/// A [`Plot`] rendering into a ratatui `Buffer`.
///
/// Created by [`Plot::widget`]; size comes from the render area, colors go straight
/// into cell styles (the host backend owns color depth).
#[derive(Debug, Clone, Copy)]
pub struct PlotWidget<'a> {
    plot: &'a Plot<'a>,
    charset: Charset,
    theme: Theme,
}

impl PlotWidget<'_> {
    /// Sets the charset; quadrants by default. Dense tiers are explicit because the
    /// host application knows its terminal and configured font better than we do.
    #[must_use]
    pub fn charset(mut self, charset: Charset) -> Self {
        self.charset = charset;
        self
    }

    /// Sets the theme (palette) used for unstyled layers.
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl Widget for PlotWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        (&self).render(area, buffer);
    }
}

impl Widget for &PlotWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let frame = Frame {
            width: area.width as usize,
            height: area.height as usize,
            charset: self.charset,
            // The mode only governs string encoding, which the adapter bypasses.
            color: ColorMode::TrueColor,
            theme: self.theme,
        };
        let surface = self.plot.rasterize(&frame);
        for (column, row, glyph, foreground, background) in surface.cells() {
            let x = area.x + column as u16;
            let y = area.y + row as u16;
            if x >= area.right() || y >= area.bottom() {
                continue;
            }
            let cell = &mut buffer[(x, y)];
            let mut symbol = [0u8; 4];
            cell.set_symbol(glyph.encode_utf8(&mut symbol));
            if let Some(fg) = convert(foreground) {
                cell.set_fg(fg);
            }
            if let Some(bg) = convert(background) {
                cell.set_bg(bg);
            }
        }
    }
}

/// Our color into ratatui's; `Default` keeps the cell's existing style.
fn convert(color: Color) -> Option<RatColor> {
    Some(match color {
        Color::Default => return None,
        Color::Black => RatColor::Black,
        Color::Red => RatColor::Red,
        Color::Green => RatColor::Green,
        Color::Yellow => RatColor::Yellow,
        Color::Blue => RatColor::Blue,
        Color::Magenta => RatColor::Magenta,
        Color::Cyan => RatColor::Cyan,
        Color::White => RatColor::White,
        Color::BrightBlack => RatColor::DarkGray,
        Color::BrightRed => RatColor::LightRed,
        Color::BrightGreen => RatColor::LightGreen,
        Color::BrightYellow => RatColor::LightYellow,
        Color::BrightBlue => RatColor::LightBlue,
        Color::BrightMagenta => RatColor::LightMagenta,
        Color::BrightCyan => RatColor::LightCyan,
        Color::BrightWhite => RatColor::White,
        Color::Ansi256(index) => RatColor::Indexed(index),
        Color::Rgb(r, g, b) => RatColor::Rgb(r, g, b),
    })
}

#[cfg(test)]
#[path = "adapter_tests.rs"]
mod tests;
