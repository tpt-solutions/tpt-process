<!-- Thank you for contributing! See CONTRIBUTING.md for the full standards. -->

## Summary

<!-- What does this PR change and why? -->

## Changes

-

## Verification & Validation

<!-- Engineering software needs evidence. What did you verify and how? -->

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace --all-features` (including doctests)
- [ ] `cargo deny check licenses bans`
- [ ] New physics/numerics covered by verification tests against analytical
      limits or golden data (state source + tolerance below)
- [ ] Determinism preserved (no iteration-order-dependent results)

## DCO sign-off

- [ ] All commits are signed off (`git commit --signoff`), certifying the
      [Developer Certificate of Origin](https://developercertificate.org/).

## Related issues / RFCs

<!-- Fixes #... ; RFC reference for new unit operations / models. -->
