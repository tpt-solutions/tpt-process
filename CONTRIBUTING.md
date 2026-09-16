# Contributing to tpt-process

Thank you for helping build open process engineering! This document explains
how to submit changes and the quality bar every contribution must meet.

## Code of Conduct

By participating you agree to abide by the
[Code of Conduct](CODE_OF_CONDUCT.md).

## Getting Started

1. Fork the repository and create a feature branch from `master`:
   ```sh
   git checkout -b feature/nrtl-activity
   ```
2. Install the toolchain (Rust 1.84+; `rustup` is recommended):
   ```sh
   rustup component add rustfmt clippy
   cargo install cargo-deny --locked
   ```
3. Make your change **with tests** (see Verification & Validation below).
4. Run the full local gate:
   ```sh
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo deny check licenses
   ```
5. Submit a pull request with a DCO sign-off (below).

## Developer Certificate of Origin (DCO)

Contributions are CLA-free: you certify ownership via `Signed-off-by` on
every commit (this is enforced by CI):

```sh
git commit --signoff
```

The sign-off asserts you have the right to submit the work under the
project's MIT OR Apache-2.0 license, per the
[DCO 1.1](https://developercertificate.org/).

## Engineering Standards

- **License chain:** every dependency must be MIT, Apache-2.0, BSD, ISC,
  Zlib, or Unicode-3.0. Copyleft is rejected by `cargo deny` (see
  `deny.toml`).
- **No panics in library code.** Solvers return `Result` with typed errors.
  `panic!`, `unwrap`, and `expect` are allowed only in tests and only behind
  `#[cfg(test)]`.
- **Units:** all public quantities are SI (K, Pa, J, mol, m, s) unless the
  function name or type says otherwise. Document any exception.
- **Determinism:** solvers must produce bit-identical results for identical
  inputs (same platform) — no dictionary-iteration order effects. Use
  ordered maps (`BTreeMap`, `Vec`) for anything entering the math.
- **`no_std`-friendliness:** core numeric code must not require allocation
  or OS services; allocation is allowed in the orchestration crates.
- **Documentation:** every public item gets a doc comment with SI units.
  Examples in docs must compile (`cargo test --doc`).

## Verification & Validation (V&V)

This is engineering software: wrong numbers are the worst possible bug.

- Every physical-property correlation and solver ships with **verification
  tests** against analytical limits (e.g., Hagen-Poiseuille for laminar pipe
  flow, ideal-gas limits of equations of state).
- Model outputs are compared to **golden reference data** in
  `test-data/golden/`, sourced from DIPPR, NIST, AIChE benchmarks, API
  standards, and ASHRAE. State the source and tolerance of every golden
  comparison in the test file.
- Numerical methods need convergence tests: iterate-until-tolerance must
  actually converge, must report non-convergence honestly (`converged:
  false`, `Err`), and must not silently clamp their way to an answer.
- New unit operations or thermodynamic models require an
  [RFC](rfcs/) before the implementation PR (see below).

## RFC Process

New unit operations, thermodynamic models, or public-API changes to published
crates require an RFC in `rfcs/` (numbered `NNNN-short-name.md`) covering
motivation, detailed design, alternatives, and unresolved questions. Small
bug fixes and internal refactors do not need one.

## Release Cadence

Releases are cut every 6 weeks from `master`. Versioning is SemVer; crates
publish in dependency order (leaf crates first). The release workflow is
`.github/workflows/release.yml`.

## Review & Merge

- Two approvals are required for RFC-scale changes; one for fixes.
- CI must be green: fmt, clippy (`-D warnings`), tests, license check, and
  docs build.
- Squash-merge with a conventional-commit message
  (`feat:`, `fix:`, `docs:`, `perf:`, `refactor:`).
