//! ANSI to HTML: the `String` a terminal would receive, decoded into colored
//! spans the way a terminal emulator would show it. The named colors take the
//! xterm defaults. This is how the site shows a color mode honestly — the
//! encoder's own SGR bytes, drawn — since the cards always carry RGB.

use crate::highlight::escape;

/// xterm's default rendering of the sixteen ANSI colors.
const NAMED: [&str; 16] = [
    "#000000", "#cd0000", "#00cd00", "#cdcd00", "#0000ee", "#cd00cd", "#00cdcd", "#e5e5e5",
    "#7f7f7f", "#ff0000", "#00ff00", "#ffff00", "#5c5cff", "#ff00ff", "#00ffff", "#ffffff",
];

fn ansi256(index: u8) -> String {
    if index < 16 {
        return NAMED[index as usize].to_string();
    }
    if index >= 232 {
        let level = 8 + (u32::from(index) - 232) * 10;
        return format!("rgb({level},{level},{level})");
    }
    let cube = u32::from(index) - 16;
    let ramp = [0, 95, 135, 175, 215, 255];
    let r = ramp[(cube / 36 % 6) as usize];
    let g = ramp[(cube / 6 % 6) as usize];
    let b = ramp[(cube % 6) as usize];
    format!("rgb({r},{g},{b})")
}

#[derive(Default, Clone, PartialEq, Eq)]
struct Style {
    foreground: Option<String>,
    background: Option<String>,
    bold: bool,
}

impl Style {
    fn css(&self) -> String {
        let mut parts = Vec::new();
        if let Some(color) = &self.foreground {
            parts.push(format!("color:{color}"));
        }
        if let Some(color) = &self.background {
            parts.push(format!("background:{color}"));
        }
        if self.bold {
            parts.push("font-weight:600".to_string());
        }
        parts.join(";")
    }

    fn apply(&mut self, codes: &[u16]) {
        let mut i = 0;
        while i < codes.len() {
            match codes[i] {
                0 => *self = Style::default(),
                1 => self.bold = true,
                22 => self.bold = false,
                30..=37 => self.foreground = Some(NAMED[(codes[i] - 30) as usize].to_string()),
                90..=97 => self.foreground = Some(NAMED[(codes[i] - 90 + 8) as usize].to_string()),
                40..=47 => self.background = Some(NAMED[(codes[i] - 40) as usize].to_string()),
                100..=107 => {
                    self.background = Some(NAMED[(codes[i] - 100 + 8) as usize].to_string())
                }
                39 => self.foreground = None,
                49 => self.background = None,
                38 | 48 => {
                    let target = if codes[i] == 38 {
                        &mut self.foreground
                    } else {
                        &mut self.background
                    };
                    match codes.get(i + 1) {
                        Some(5) => {
                            *target = codes.get(i + 2).map(|&n| ansi256(n as u8));
                            i += 2;
                        }
                        Some(2) => {
                            if let (Some(&r), Some(&g), Some(&b)) =
                                (codes.get(i + 2), codes.get(i + 3), codes.get(i + 4))
                            {
                                *target = Some(format!("rgb({r},{g},{b})"));
                            }
                            i += 4;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
}

/// Decodes SGR-colored text into HTML spans; every other escape is dropped.
pub fn to_html(ansi: &str) -> String {
    let mut out = String::with_capacity(ansi.len());
    let mut style = Style::default();
    let mut open = false;
    let chars: Vec<char> = ansi.chars().collect();
    let mut i = 0;
    let mut text = String::new();
    let flush = |out: &mut String, text: &mut String, style: &Style, open: &mut bool| {
        if text.is_empty() {
            return;
        }
        let css = style.css();
        if css.is_empty() {
            out.push_str(&escape(text));
        } else {
            out.push_str("<span style=\"");
            out.push_str(&css);
            out.push_str("\">");
            out.push_str(&escape(text));
            out.push_str("</span>");
        }
        let _ = open;
        text.clear();
    };
    while i < chars.len() {
        if chars[i] == '\u{1b}' && chars.get(i + 1) == Some(&'[') {
            let mut end = i + 2;
            while end < chars.len() && !chars[end].is_ascii_alphabetic() {
                end += 1;
            }
            let body: String = chars[i + 2..end.min(chars.len())].iter().collect();
            if chars.get(end) == Some(&'m') {
                flush(&mut out, &mut text, &style, &mut open);
                let codes: Vec<u16> = body
                    .split(';')
                    .filter_map(|code| code.parse().ok())
                    .collect();
                if codes.is_empty() {
                    style = Style::default();
                } else {
                    style.apply(&codes);
                }
            }
            i = end + 1;
            continue;
        }
        text.push(chars[i]);
        i += 1;
    }
    flush(&mut out, &mut text, &style, &mut open);
    out
}
