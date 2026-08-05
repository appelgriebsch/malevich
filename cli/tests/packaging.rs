//! Guards the hand-written packaging (shell completions, man page) against drift:
//! every chart the CLI ships must appear in each artifact, so a new subcommand
//! cannot land without its completion and man entries.

const BASH: &str = include_str!("../completions/kaz.bash");
const ZSH: &str = include_str!("../completions/kaz.zsh");
const FISH: &str = include_str!("../completions/kaz.fish");
const MAN: &str = include_str!("../man/kaz.1");

/// Every chart subcommand `kaz` accepts.
const CHARTS: [&str; 11] = [
    "line", "scatter", "bar", "hist", "count", "density", "ecdf", "box", "violin", "hist2d",
    "heatmap",
];

#[test]
fn completions_list_every_chart() {
    for (name, text) in [("bash", BASH), ("zsh", ZSH), ("fish", FISH)] {
        for chart in CHARTS {
            assert!(
                text.contains(chart),
                "the {name} completion is missing `{chart}`"
            );
        }
    }
}

#[test]
fn the_man_page_documents_every_chart() {
    for chart in CHARTS {
        assert!(MAN.contains(chart), "the man page is missing `{chart}`");
    }
}

#[test]
fn packaging_covers_the_value_flag_choices() {
    // The enumerated flag values must stay in sync too — a missing charset tier or
    // color mode is a silent completion gap.
    for choice in ["braille", "octant", "sextant", "always", "never", "xyxy"] {
        assert!(BASH.contains(choice), "bash completion missing `{choice}`");
        assert!(ZSH.contains(choice), "zsh completion missing `{choice}`");
        assert!(FISH.contains(choice), "fish completion missing `{choice}`");
    }
}
