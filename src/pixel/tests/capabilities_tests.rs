use super::{Source, probing_is_safe, resolve};
use crate::pixel::probe::Report;
use crate::pixel::{Capabilities, Protocol};

fn answered(report: Report) -> Report {
    Report {
        answered: true,
        ..report
    }
}

#[test]
fn a_kitty_probe_answer_is_ground_truth() {
    let report = answered(Report {
        kitty: true,
        terminal: Some("kitty(0.40.1)".into()),
        cell_size: Some((22, 44)),
        ..Report::default()
    });
    let capabilities = resolve(Some(&report), Vec::new(), None);
    assert_eq!(capabilities.protocols, [Protocol::Kitty]);
    assert_eq!(capabilities.cell_size, Some((22, 44)));
    assert_eq!(capabilities.source, Source::Probed);
}

#[test]
fn an_iterm2_probe_answer_yields_its_protocol_ladder() {
    let report = answered(Report {
        terminal: Some("iTerm2 3.5.9".into()),
        sixel: true,
        cell_size: Some((15, 32)),
        ..Report::default()
    });
    let capabilities = resolve(Some(&report), Vec::new(), None);
    assert_eq!(capabilities.protocols, [Protocol::ITerm2, Protocol::Sixel]);
    assert_eq!(capabilities.source, Source::Probed);
}

#[test]
fn probe_and_sniff_merge_into_one_ranked_answer() {
    // The probe proved sixel but XTVERSION went unanswered; the environment
    // still names iTerm2. Union, ranked: native protocol first.
    let report = answered(Report {
        sixel: true,
        ..Report::default()
    });
    let sniffed = vec![Protocol::ITerm2, Protocol::Sixel];
    let capabilities = resolve(Some(&report), sniffed, Some((8, 16)));
    assert_eq!(capabilities.protocols, [Protocol::ITerm2, Protocol::Sixel]);
    assert_eq!(capabilities.source, Source::Probed);
    assert_eq!(capabilities.cell_size, Some((8, 16)));
}

#[test]
fn an_unanswered_probe_is_not_evidence_and_falls_back_to_sniffing() {
    let unanswered = Report::default();
    let capabilities = resolve(Some(&unanswered), vec![Protocol::Kitty], None);
    assert_eq!(capabilities.protocols, [Protocol::Kitty]);
    assert_eq!(capabilities.source, Source::Sniffed);
}

#[test]
fn a_probed_cell_size_outranks_the_ioctl_fallback() {
    let report = answered(Report {
        sixel: true,
        cell_size: Some((11, 21)),
        ..Report::default()
    });
    let capabilities = resolve(Some(&report), Vec::new(), Some((8, 16)));
    assert_eq!(capabilities.cell_size, Some((11, 21)));
}

#[test]
fn best_takes_the_first_protocol_at_the_known_cell_size() {
    let capabilities = resolve(
        Some(&answered(Report {
            kitty: true,
            sixel: true,
            cell_size: Some((10, 20)),
            ..Report::default()
        })),
        Vec::new(),
        None,
    );
    let graphics = capabilities.best().expect("kitty is available");
    assert_eq!(graphics.protocol, Protocol::Kitty);
    assert_eq!(graphics.cell_size, (10, 20));
}

#[test]
fn best_is_none_when_cells_are_the_ceiling() {
    let capabilities = resolve(None, Vec::new(), Some((8, 16)));
    assert_eq!(capabilities.protocols, []);
    assert_eq!(capabilities.best(), None);
}

#[test]
fn capabilities_without_a_cell_size_keep_the_default_in_best() {
    let capabilities = Capabilities {
        protocols: vec![Protocol::Sixel],
        cell_size: None,
        source: Source::Sniffed,
    };
    assert_eq!(capabilities.best().expect("sixel").cell_size, (8, 16));
}

#[test]
fn probe_safety_is_keyed_to_the_output_destination() {
    let clear = |_: &str| None;
    assert!(probing_is_safe(true, &clear));
    assert!(!probing_is_safe(false, &clear));

    let behind_tmux = |name: &str| (name == "TMUX").then(|| "session".into());
    assert!(!probing_is_safe(true, &behind_tmux));
}
