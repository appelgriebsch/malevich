//! Stdin framing: text into rows of fields (D-C5).
//!
//! The default separator is any run of whitespace, so bare numbers-per-line, TSV,
//! and `column`-style output all just work. `-d CHAR` sets one explicit separator
//! (`-d,` for CSV-shaped data) and then does not collapse runs — an empty field
//! between two delimiters is a real, if unparseable, field. No quoting, no escaping,
//! no multi-char delimiters: this parses *fields*, not CSV. Real CSV goes through
//! `xsv`/`mlr` upstream.

/// A framed input: optional header names plus the data rows, each a list of raw
/// string fields. Rows may be ragged; the series layer squares them up.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Table {
    pub header: Option<Vec<String>>,
    pub rows: Vec<Vec<String>>,
}

/// Frames `text` into a [`Table`]. Blank lines are skipped everywhere. With
/// `header`, the first non-blank line supplies column names.
pub fn frame(text: &str, delimiter: Option<char>, header: bool) -> Table {
    let mut lines = text
        .lines()
        .map(|line| split(line, delimiter))
        .filter(|fields| !fields.is_empty());

    let header = if header { lines.next() } else { None };
    Table {
        header,
        rows: lines.collect(),
    }
}

/// Splits one line into fields. Whitespace-run by default; on a fixed delimiter
/// the split is literal (runs are not collapsed), but a line that is entirely
/// blank still yields no fields.
fn split(line: &str, delimiter: Option<char>) -> Vec<String> {
    match delimiter {
        None => line.split_whitespace().map(str::to_owned).collect(),
        Some(sep) => {
            if line.trim().is_empty() {
                Vec::new()
            } else {
                line.split(sep).map(str::to_owned).collect()
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/input_tests.rs"]
mod tests;
