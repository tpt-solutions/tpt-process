# Introduction

`tpt-process` is a fully open-source, MIT-licensed computational engine for
chemical and process engineering: thermodynamics, fluid flow, heat
transfer, separations, reaction engineering, and flowsheet simulation.

## Why?

Commercial process simulators cost $30k–$100k per seat and lock property
data behind recurring fees. `tpt-process` breaks those locks with pure
Rust, an open built-in databank, WebAssembly bindings, and a strict
MIT OR Apache-2.0 dependency chain enforced by `cargo deny`.

## Design commitments

- **Deterministic**: identical inputs give bit-identical results.
- **Honest**: non-convergence is an error, never a silent fallback.
- **SI units** everywhere unless a name says otherwise.
- **No panics** in library code.
