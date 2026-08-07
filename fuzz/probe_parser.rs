//! Dependency-free deterministic fuzz runner for the terminal reply parser.
//!
//! Compile directly so this remains usable without a cargo-fuzz installation:
//! `rustc --edition=2024 -O fuzz/probe_parser.rs -o target/probe-parser-fuzz`.

#[allow(dead_code)]
#[path = "../src/pixel/probe.rs"]
mod probe;

const KNOWN_REPLY: &[u8] = b"noise\x1b_Gi=31;OK\x1b\\\x1bP>|terminal\x1b\\\x1b[6;18;9t\x1b[?62;4c";

fn main() {
    let cases = std::env::args()
        .nth(1)
        .map(|value| value.parse().expect("case count must be an integer"))
        .unwrap_or(100_000usize);
    let mut state = std::env::args()
        .nth(2)
        .map(|value| value.parse().expect("seed must be an integer"))
        .unwrap_or(0x4d41_4c45_5649_4348_u64);

    for case in 0..cases {
        let length = next(&mut state) as usize % (probe::MAX_REPLY_BYTES + 1);
        let mut bytes = vec![0; length];
        for byte in &mut bytes {
            *byte = next(&mut state) as u8;
        }
        mutate_with_known_reply(case, &mut state, &mut bytes);
        exercise(case, &bytes);
    }

    for end in 0..=KNOWN_REPLY.len() {
        exercise(cases + end, &KNOWN_REPLY[..end]);
    }
}

fn mutate_with_known_reply(case: usize, state: &mut u64, bytes: &mut [u8]) {
    if case % 2 != 0 || bytes.is_empty() {
        return;
    }
    let copied = KNOWN_REPLY.len().min(bytes.len());
    let start = next(state) as usize % (bytes.len() - copied + 1);
    bytes[start..start + copied].copy_from_slice(&KNOWN_REPLY[..copied]);
    for _ in 0..case % 7 {
        let index = next(state) as usize % bytes.len();
        bytes[index] = next(state) as u8;
    }
}

fn exercise(case: usize, bytes: &[u8]) {
    let report = probe::parse(bytes);
    assert_eq!(
        report.answered,
        probe::done(bytes),
        "barrier disagreement in generated case {case}"
    );
}

fn next(state: &mut u64) -> u64 {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545_f491_4f6c_dd1d)
}
