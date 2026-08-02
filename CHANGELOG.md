# Changelog

Notable changes, written for humans. Pre-1.0, breaking changes are expected and listed
without apology.

## Unreleased

- Tick placement (`scale::Ticks`): extended Wilkinson (Talbot–Lin–Hanrahan) with
  exact-decimal labels — labels parse back to their values, share one fraction width
  per axis, and never show float artifacts. Placement runs in microseconds.
- Project scaffold: crate skeleton, terminology contract, CI.
