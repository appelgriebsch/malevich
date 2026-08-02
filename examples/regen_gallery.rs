//! Regenerates `EXAMPLES.md` from the gallery examples, or verifies it is current.
//!
//! The gallery is both the showcase and the system test: every example renders a
//! fixed `Frame::plain`, so its output is deterministic, and CI runs this with
//! `--check` to fail on any stale gallery.

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
];

const PATH: &str = "EXAMPLES.md";

fn main() {
    let check = std::env::args().any(|argument| argument == "--check");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    let mut content = String::from(
        "# Gallery\n\n\
         The showcase and the system test in one artifact. Regenerate with\n\
         `cargo run --example regen_gallery`; CI fails when this file is stale.\n\
         Every example renders a fixed `Frame::plain`, so output is deterministic.\n",
    );
    for (name, story) in GALLERY {
        let output = Command::new(&cargo)
            .args(["run", "--quiet", "--example", name])
            .output()
            .expect("failed to run cargo");
        assert!(
            output.status.success(),
            "example {name} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8(output.stdout).expect("example output is not UTF-8");
        content.push_str(&format!(
            "\n## {name}\n\n{story}\nSource: [examples/{name}.rs](examples/{name}.rs)\n\n\
             ```text\n{}\n```\n",
            text.trim_end_matches('\n')
        ));
    }

    if check {
        let existing = std::fs::read_to_string(PATH).unwrap_or_default();
        if existing != content {
            eprintln!("{PATH} is stale; run: cargo run --example regen_gallery");
            std::process::exit(1);
        }
        println!("{PATH} is current.");
    } else {
        std::fs::write(PATH, content).expect("failed to write the gallery");
        println!("{PATH} regenerated.");
    }
}
