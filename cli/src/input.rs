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

/// Resolves one `--cols`/`--by` selector: a 0-based index, or a header name.
/// The failure text is actionable — it lists the available names or says why
/// there are none.
pub fn column_index(table: &Table, selector: &str) -> Result<usize, String> {
    if let Ok(index) = selector.parse::<usize>() {
        return Ok(index);
    }
    match &table.header {
        Some(names) => names
            .iter()
            .position(|name| name == selector)
            .ok_or_else(|| {
                format!(
                    "no column named `{selector}`; the header has: {}",
                    names.join(", ")
                )
            }),
        None => Err(format!(
            "no column named `{selector}`: selecting by name needs a header row (-H); \
             0-based indices always work"
        )),
    }
}

/// Projects the table onto `selectors`, in selector order — the whole-table
/// transform behind `--cols`. Short rows contribute empty fields.
pub fn select(table: &Table, selectors: &[String]) -> Result<Table, String> {
    let indices = selectors
        .iter()
        .map(|selector| column_index(table, selector))
        .collect::<Result<Vec<_>, _>>()?;
    let project = |row: &Vec<String>| -> Vec<String> {
        indices
            .iter()
            .map(|&index| row.get(index).cloned().unwrap_or_default())
            .collect()
    };
    Ok(Table {
        header: table.header.as_ref().map(project),
        rows: table.rows.iter().map(project).collect(),
    })
}

/// One raw column as strings — the categorical shape for `--by`. Fields
/// missing on short rows become `-`, a visible category rather than a silent
/// drop.
pub fn string_column(table: &Table, index: usize) -> Vec<String> {
    table
        .rows
        .iter()
        .map(|row| {
            let field = row.get(index).map(String::as_str).unwrap_or("");
            if field.is_empty() { "-" } else { field }.to_owned()
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/input_tests.rs"]
mod tests;
