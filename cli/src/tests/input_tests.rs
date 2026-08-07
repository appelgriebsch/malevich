use super::*;

#[test]
fn whitespace_runs_are_one_separator() {
    let table = frame("1   2\t3\n4 5 6\n", None, false);
    assert_eq!(table.header, None);
    assert_eq!(
        table.rows,
        vec![
            vec!["1".to_string(), "2".into(), "3".into()],
            vec!["4".into(), "5".into(), "6".into()],
        ]
    );
}

#[test]
fn bare_numbers_per_line_are_single_field_rows() {
    let table = frame("1\n4\n2\n", None, false);
    assert_eq!(table.rows, vec![vec!["1"], vec!["4"], vec!["2"]]);
}

#[test]
fn a_fixed_delimiter_preserves_empty_fields() {
    let table = frame("a,,b\n1,2,3\n", Some(','), false);
    assert_eq!(
        table.rows,
        vec![
            vec!["a".to_string(), "".into(), "b".into()],
            vec!["1".into(), "2".into(), "3".into()],
        ]
    );
}

#[test]
fn blank_lines_are_skipped_everywhere() {
    let table = frame("\n1 2\n\n  \n3 4\n", None, false);
    assert_eq!(table.rows, vec![vec!["1", "2"], vec!["3", "4"]]);
}

#[test]
fn a_header_consumes_the_first_nonblank_row() {
    let table = frame("\nstep loss\n0 4\n1 2\n", None, true);
    assert_eq!(table.header, Some(vec!["step".to_string(), "loss".into()]));
    assert_eq!(table.rows, vec![vec!["0", "4"], vec!["1", "2"]]);
}

#[test]
fn no_header_flag_leaves_the_first_row_as_data() {
    let table = frame("step loss\n0 4\n", None, false);
    assert_eq!(table.header, None);
    assert_eq!(table.rows.len(), 2);
}

#[test]
fn literal_delimiters_preserve_boundaries_for_ascii_unicode_and_nul() {
    for separator in [',', '|', '\u{1f9ea}', '\0'] {
        let text = format!("left{separator}{separator}right\n{separator}\n");
        let table = frame(&text, Some(separator), false);
        assert_eq!(table.rows[0], ["left", "", "right"]);
        assert_eq!(table.rows[1], ["", ""]);
    }
}

#[test]
fn crlf_and_lf_inputs_frame_identically() {
    assert_eq!(
        frame("name,value\r\na,1\r\nb,2\r\n", Some(','), true),
        frame("name,value\na,1\nb,2\n", Some(','), true)
    );
}
