use super::{MAX_REPLY_BYTES, Report, done, parse};

#[test]
fn a_kitty_terminal_answers_the_graphics_query_and_da1() {
    // kitty: graphics OK, XTVERSION, cell size, DA1 without sixel.
    let replies = b"\x1b_Gi=31;OK\x1b\\\x1bP>|kitty(0.40.1)\x1b\\\x1b[6;44;22t\x1b[?62;c";
    assert!(done(replies));
    let report = parse(replies);
    assert!(report.kitty);
    assert!(!report.sixel);
    assert_eq!(report.terminal.as_deref(), Some("kitty(0.40.1)"));
    assert_eq!(report.cell_size, Some((22, 44)));
    assert!(report.answered);
}

#[test]
fn an_iterm2_terminal_reports_name_sixel_and_cell_size() {
    let replies = b"\x1bP>|iTerm2 3.5.9\x1b\\\x1b[?1;0;1024S\x1b[6;32;15t\x1b[?62;4c";
    let report = parse(replies);
    assert!(!report.kitty);
    assert!(report.sixel);
    assert_eq!(report.terminal.as_deref(), Some("iTerm2 3.5.9"));
    assert_eq!(report.cell_size, Some((15, 32)));
    assert!(report.answered);
}

#[test]
fn an_xterm_with_sixel_advertises_it_in_da1_and_xtsmgraphics() {
    let replies = b"\x1b[?1;0;334S\x1b[?64;1;2;4;6;9;15;18;21;22c";
    assert!(done(replies));
    let report = parse(replies);
    assert!(report.sixel);
    assert!(report.answered);
    assert_eq!(report.terminal, None);
    assert_eq!(report.cell_size, None);
}

#[test]
fn a_failed_xtsmgraphics_status_is_not_sixel_evidence() {
    // Status 2: error. DA1 without attribute 4 either.
    let report = parse(b"\x1b[?1;2;0S\x1b[?62;22c");
    assert!(!report.sixel);
    assert!(report.answered);
}

#[test]
fn silence_reports_nothing_and_is_not_done() {
    assert!(!done(b""));
    assert_eq!(parse(b""), Report::default());
    // Junk and partial sequences neither crash nor count.
    assert!(!done(b"garbage\x1b[?62;"));
    assert_eq!(parse(b"garbage\x1b[?62;"), Report::default());
}

#[test]
fn a_hairline_cell_size_is_rejected_as_nonsense() {
    let report = parse(b"\x1b[6;1;1t\x1b[?62;c");
    assert_eq!(report.cell_size, None);
}

#[test]
fn interleaved_unknown_replies_are_skipped() {
    // A cursor-position report and an unrelated DCS between real answers.
    let replies = b"\x1b[24;80R\x1bP1$r0m\x1b\\\x1b_Gi=31;OK\x1b\\\x1b[?62;4c";
    let report = parse(replies);
    assert!(report.kitty);
    assert!(report.sixel);
    assert!(report.answered);
}

#[test]
fn every_reply_prefix_is_safe_and_agrees_with_the_barrier_parser() {
    let replies = b"noise\x1b_Gi=31;OK\x1b\\\x1bP>|terminal\x1b\\\x1b[6;18;9t\x1b[?62;4c";
    for end in 0..=replies.len() {
        let prefix = &replies[..end];
        assert_eq!(parse(prefix).answered, done(prefix), "prefix length {end}");
    }

    // Every possible byte value, including invalid UTF-8 and a lone ESC, is
    // ordinary untrusted input to the pure parser.
    let bytes: Vec<u8> = (u8::MIN..=u8::MAX).collect();
    for end in 0..=bytes.len() {
        let _ = parse(&bytes[..end]);
        let _ = done(&bytes[..end]);
    }
}

#[test]
fn deterministic_arbitrary_streams_are_total_and_barrier_consistent() {
    let mut state = 0x4d41_4c45_5649_4348_u64;
    let known = b"\x1b_Gi=31;OK\x1b\\\x1bP>|terminal\x1b\\\x1b[6;18;9t\x1b[?62;4c";

    for case in 0..10_000usize {
        let length = next(&mut state) as usize % (MAX_REPLY_BYTES + 1);
        let mut bytes = vec![0; length];
        for byte in &mut bytes {
            *byte = next(&mut state) as u8;
        }
        // Half the corpus carries a mutated or truncated real reply, so the
        // state machine sees much more than uniformly rare ESC bytes.
        if case % 2 == 0 && !bytes.is_empty() {
            let copied = known.len().min(bytes.len());
            let start = next(&mut state) as usize % (bytes.len() - copied + 1);
            bytes[start..start + copied].copy_from_slice(&known[..copied]);
            for _ in 0..case % 5 {
                let index = next(&mut state) as usize % bytes.len();
                bytes[index] = next(&mut state) as u8;
            }
        }

        let report = parse(&bytes);
        assert_eq!(
            report.answered,
            done(&bytes),
            "barrier disagreement in generated case {case}"
        );
    }
}

fn next(state: &mut u64) -> u64 {
    // xorshift64*: deterministic, dependency-free input generation rather than
    // a source of statistical randomness.
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545_f491_4f6c_dd1d)
}
