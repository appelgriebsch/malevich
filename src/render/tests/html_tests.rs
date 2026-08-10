use super::escape;

#[test]
fn glyphs_are_escaped_as_html_content() {
    let mut html = String::new();
    for glyph in "<&>\"'".chars() {
        escape(glyph, &mut html);
    }
    assert_eq!(html, "&lt;&amp;&gt;\"'");
}
