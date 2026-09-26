//! A small syntax highlighter for the languages the docs quote: Rust,
//! JavaScript and TypeScript, shell, JSON, and TOML. Tokens become `<span>`s
//! with one-letter classes; anything it does not recognize passes through
//! escaped. Owned craft rather than a grammar bundle: the docs need six
//! languages and the terminal needs none of them.

/// Escapes text for HTML element content.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

/// Highlights `code` written in `language` (a fence tag) as HTML.
pub fn highlight(language: &str, code: &str) -> String {
    match language {
        "rust" | "rs" => rust(code),
        "js" | "javascript" | "ts" | "typescript" | "tsx" | "jsx" => script(code),
        "sh" | "bash" | "shell" | "zsh" | "console" => shell(code),
        "json" => json(code),
        "toml" => toml(code),
        _ => escape(code),
    }
}

const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];

const SCRIPT_KEYWORDS: &[&str] = &[
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "default",
    "delete",
    "do",
    "else",
    "export",
    "extends",
    "finally",
    "for",
    "from",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "let",
    "new",
    "null",
    "of",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "false",
    "try",
    "typeof",
    "undefined",
    "var",
    "void",
    "while",
    "yield",
    "interface",
    "type",
    "implements",
    "readonly",
    "as",
];

fn span(out: &mut String, class: &str, text: &str) {
    out.push_str("<span class=\"");
    out.push_str(class);
    out.push_str("\">");
    out.push_str(&escape(text));
    out.push_str("</span>");
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Reads a string literal starting at `start` (which holds the quote), with
/// backslash escapes, and returns the index one past its close.
fn string_end(chars: &[char], start: usize, quote: char) -> usize {
    let mut i = start + 1;
    while i < chars.len() {
        if chars[i] == '\\' {
            i += 2;
            continue;
        }
        if chars[i] == quote {
            return i + 1;
        }
        i += 1;
    }
    chars.len()
}

fn number_end(chars: &[char], start: usize) -> usize {
    let mut i = start;
    while i < chars.len()
        && (chars[i].is_ascii_alphanumeric()
            || chars[i] == '_'
            || chars[i] == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
    {
        i += 1;
    }
    i
}

fn rust(code: &str) -> String {
    let chars: Vec<char> = code.chars().collect();
    let mut out = String::with_capacity(code.len() * 2);
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        // Comments.
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            let end = chars[i..]
                .iter()
                .position(|&c| c == '\n')
                .map_or(chars.len(), |n| i + n);
            span(&mut out, "c", &chars[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        // Attributes.
        if c == '#' && matches!(chars.get(i + 1), Some('[') | Some('!')) {
            let end = chars[i..]
                .iter()
                .position(|&c| c == ']')
                .map_or(chars.len(), |n| i + n + 1);
            span(&mut out, "a", &chars[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        // Strings (plain and raw, one hash deep).
        if c == '"' {
            let end = string_end(&chars, i, '"');
            span(&mut out, "s", &chars[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        if c == 'r' && chars.get(i + 1) == Some(&'#') && chars.get(i + 2) == Some(&'"') {
            let mut end = i + 3;
            while end + 1 < chars.len() && !(chars[end] == '"' && chars[end + 1] == '#') {
                end += 1;
            }
            let end = (end + 2).min(chars.len());
            span(&mut out, "s", &chars[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        // Lifetimes and char literals.
        if c == '\'' {
            if chars.get(i + 2) == Some(&'\'')
                || (chars.get(i + 1) == Some(&'\\') && chars.get(i + 3) == Some(&'\''))
            {
                let end = if chars.get(i + 1) == Some(&'\\') {
                    i + 4
                } else {
                    i + 3
                };
                span(&mut out, "s", &chars[i..end].iter().collect::<String>());
                i = end;
                continue;
            }
            let mut end = i + 1;
            while end < chars.len() && is_ident(chars[end]) {
                end += 1;
            }
            span(&mut out, "l", &chars[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        if c.is_ascii_digit() {
            let end = number_end(&chars, i);
            span(&mut out, "n", &chars[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        if is_ident_start(c) {
            let mut end = i + 1;
            while end < chars.len() && is_ident(chars[end]) {
                end += 1;
            }
            let word: String = chars[i..end].iter().collect();
            if chars.get(end) == Some(&'!') {
                span(&mut out, "m", &format!("{word}!"));
                i = end + 1;
            } else if RUST_KEYWORDS.contains(&word.as_str()) {
                span(&mut out, "k", &word);
                i = end;
            } else if word.chars().next().is_some_and(char::is_uppercase) {
                span(&mut out, "t", &word);
                i = end;
            } else if chars.get(end) == Some(&'(') {
                span(&mut out, "f", &word);
                i = end;
            } else {
                out.push_str(&escape(&word));
                i = end;
            }
            continue;
        }
        out.push_str(&escape(&c.to_string()));
        i += 1;
    }
    out
}

fn script(code: &str) -> String {
    let chars: Vec<char> = code.chars().collect();
    let mut out = String::with_capacity(code.len() * 2);
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            let end = chars[i..]
                .iter()
                .position(|&c| c == '\n')
                .map_or(chars.len(), |n| i + n);
            span(&mut out, "c", &chars[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        if c == '/' && chars.get(i + 1) == Some(&'*') {
            let mut end = i + 2;
            while end + 1 < chars.len() && !(chars[end] == '*' && chars[end + 1] == '/') {
                end += 1;
            }
            let end = (end + 2).min(chars.len());
            span(&mut out, "c", &chars[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        if c == '"' || c == '\'' || c == '`' {
            let end = string_end(&chars, i, c);
            span(&mut out, "s", &chars[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        if c.is_ascii_digit() {
            let end = number_end(&chars, i);
            span(&mut out, "n", &chars[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        if is_ident_start(c) || c == '$' {
            let mut end = i + 1;
            while end < chars.len() && (is_ident(chars[end]) || chars[end] == '$') {
                end += 1;
            }
            let word: String = chars[i..end].iter().collect();
            if SCRIPT_KEYWORDS.contains(&word.as_str()) {
                span(&mut out, "k", &word);
            } else if word.chars().next().is_some_and(char::is_uppercase) {
                span(&mut out, "t", &word);
            } else if chars.get(end) == Some(&'(') {
                span(&mut out, "f", &word);
            } else {
                out.push_str(&escape(&word));
            }
            i = end;
            continue;
        }
        out.push_str(&escape(&c.to_string()));
        i += 1;
    }
    out
}

fn shell(code: &str) -> String {
    let mut out = String::with_capacity(code.len() * 2);
    for (index, line) in code.split('\n').enumerate() {
        if index > 0 {
            out.push('\n');
        }
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        out.push_str(indent);
        if trimmed.starts_with('#') {
            span(&mut out, "c", trimmed);
            continue;
        }
        // The first word of each pipeline stage is a command.
        let mut command_next = true;
        let mut token = String::new();
        let chars: Vec<char> = trimmed.chars().collect();
        let mut i = 0;
        let flush = |out: &mut String, token: &mut String, command_next: &mut bool| {
            if token.is_empty() {
                return;
            }
            if *command_next {
                span(out, "f", token);
                *command_next = false;
            } else if token.starts_with('-') {
                span(out, "a", token);
            } else if token.starts_with('$') {
                span(out, "l", token);
            } else {
                out.push_str(&escape(token));
            }
            token.clear();
        };
        while i < chars.len() {
            let c = chars[i];
            match c {
                '"' | '\'' => {
                    flush(&mut out, &mut token, &mut command_next);
                    let end = string_end(&chars, i, c);
                    span(&mut out, "s", &chars[i..end].iter().collect::<String>());
                    i = end;
                    continue;
                }
                '#' if token.is_empty() => {
                    flush(&mut out, &mut token, &mut command_next);
                    span(&mut out, "c", &chars[i..].iter().collect::<String>());
                    i = chars.len();
                    continue;
                }
                '|' | ';' | '&' | '(' | ')' => {
                    flush(&mut out, &mut token, &mut command_next);
                    span(&mut out, "p", &c.to_string());
                    command_next = true;
                }
                ' ' | '\t' => {
                    flush(&mut out, &mut token, &mut command_next);
                    out.push(c);
                }
                _ => token.push(c),
            }
            i += 1;
        }
        flush(&mut out, &mut token, &mut command_next);
    }
    out
}

fn json(code: &str) -> String {
    let chars: Vec<char> = code.chars().collect();
    let mut out = String::with_capacity(code.len() * 2);
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '"' {
            let end = string_end(&chars, i, '"');
            let text: String = chars[i..end].iter().collect();
            let rest: String = chars[end..]
                .iter()
                .take_while(|c| c.is_whitespace())
                .collect();
            let is_key = chars.get(end + rest.len()) == Some(&':');
            span(&mut out, if is_key { "a" } else { "s" }, &text);
            i = end;
            continue;
        }
        if c.is_ascii_digit() || c == '-' {
            let end = number_end(&chars, i + 1).max(i + 1);
            span(&mut out, "n", &chars[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        if c.is_alphabetic() {
            let mut end = i;
            while end < chars.len() && chars[end].is_alphabetic() {
                end += 1;
            }
            span(&mut out, "k", &chars[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        out.push_str(&escape(&c.to_string()));
        i += 1;
    }
    out
}

fn toml(code: &str) -> String {
    let mut out = String::with_capacity(code.len() * 2);
    for (index, line) in code.split('\n').enumerate() {
        if index > 0 {
            out.push('\n');
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            span(&mut out, "c", line);
        } else if trimmed.starts_with('[') {
            span(&mut out, "t", line);
        } else if let Some((key, value)) = line.split_once('=') {
            span(&mut out, "a", key);
            out.push('=');
            out.push_str(&script(value));
        } else {
            out.push_str(&escape(line));
        }
    }
    out
}
