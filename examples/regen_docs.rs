//! Regenerates every chart embedded in the docs, or verifies they are current.
//!
//! Two mechanisms, one honesty rule — no chart in any markdown file is typed by a
//! human; every one is real program output:
//!
//! - `EXAMPLES.md` is built whole from the gallery examples.
//! - Any markdown file may embed `<!-- generated:NAME -->` … `<!-- /generated -->`;
//!   the block between the markers is replaced with the stdout of
//!   `cargo run --example NAME` in a `text` fence.
//!
//! CI runs this with `--check` and fails on any stale file. Examples used here must
//! render fixed `Frame::plain` frames so their output is deterministic.

use std::process::Command;

/// The gallery: example name and the one-line story it tells.
const GALLERY: &[(&str, &str)] = &[
    (
        "sine",
        "Function sampling: curves drawn from `f(x)`, one sample per subpixel column.",
    ),
    (
        "loss",
        "The training-loop story: two series on shared scales; unrecorded steps are gaps.",
    ),
    (
        "languages",
        "Categorical bars from a zero baseline, with eighth-block precision at the top.",
    ),
    (
        "clusters",
        "A labeled scatter: two point layers, named in the legend.",
    ),
    (
        "waveform",
        "Ten million points through the auto-inserted M4 aggregation — pixel-identical \
         to drawing every point, in tens of milliseconds.",
    ),
    (
        "distribution",
        "A histogram via the Bin stat: automatic bin count, nice decimal edges, \
         contiguous bars from zero.",
    ),
    (
        "powerlaw",
        "Log-log axes: power laws render straight, with decade ticks on both axes.",
    ),
    (
        "energy",
        "Stacked areas via the Stack stat: each layer sits on the sum of the ones below.",
    ),
    (
        "annotated",
        "Annotations: a Rule for the target line, a Text note at data coordinates.",
    ),
];

/// Markdown files scanned for `<!-- generated:NAME -->` blocks.
const SPLICED: &[&str] = &["README.md"];

fn main() {
    let check = std::env::args().any(|argument| argument == "--check");
    let mut stale = Vec::new();

    let gallery = gallery_content();
    apply("EXAMPLES.md", gallery, check, &mut stale);

    for path in SPLICED {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
        apply(path, splice(&content), check, &mut stale);
    }

    if check {
        if stale.is_empty() {
            println!("All generated docs are current.");
        } else {
            eprintln!(
                "Stale generated docs: {}. Run: cargo run --example regen_docs",
                stale.join(", ")
            );
            std::process::exit(1);
        }
    }
}

/// Runs one example and returns its stdout with the trailing newline trimmed.
fn output_of(name: &str) -> String {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(cargo)
        .args(["run", "--quiet", "--example", name])
        .output()
        .expect("failed to run cargo");
    assert!(
        output.status.success(),
        "example {name} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("example output is not UTF-8")
        .trim_end_matches('\n')
        .to_string()
}

fn gallery_content() -> String {
    let mut content = String::from(
        "# Gallery\n\n\
         The showcase and the system test in one artifact. Regenerate with\n\
         `cargo run --example regen_docs`; CI fails when this file is stale.\n\
         Every example renders a fixed `Frame::plain`, so output is deterministic.\n",
    );
    for (name, story) in GALLERY {
        content.push_str(&format!(
            "\n## {name}\n\n{story}\nSource: [examples/{name}.rs](examples/{name}.rs)\n\n\
             ```text\n{}\n```\n",
            output_of(name)
        ));
    }
    content
}

/// Replaces every `<!-- generated:NAME -->` block with the named example's output.
fn splice(content: &str) -> String {
    const OPEN: &str = "<!-- generated:";
    const OPEN_END: &str = " -->";
    const CLOSE: &str = "<!-- /generated -->";

    let mut result = String::with_capacity(content.len());
    let mut rest = content;
    while let Some(start) = rest.find(OPEN) {
        let (head, tail) = rest.split_at(start);
        result.push_str(head);
        let name_end = tail[OPEN.len()..]
            .find(OPEN_END)
            .expect("unterminated generated marker");
        let name = &tail[OPEN.len()..OPEN.len() + name_end];
        let marker_end = OPEN.len() + name_end + OPEN_END.len();
        result.push_str(&tail[..marker_end]);
        let close = tail.find(CLOSE).expect("missing closing generated marker");
        result.push_str(&format!("\n```text\n{}\n```\n", output_of(name)));
        rest = &tail[close..];
    }
    result.push_str(rest);
    result
}

/// Writes `content` to `path`, or in check mode records staleness.
fn apply(path: &str, content: String, check: bool, stale: &mut Vec<String>) {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    if existing == content {
        return;
    }
    if check {
        stale.push(path.to_string());
    } else {
        std::fs::write(path, content)
            .unwrap_or_else(|error| panic!("failed to write {path}: {error}"));
        println!("{path} regenerated.");
    }
}
