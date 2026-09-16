# Security Policy

## Supported Versions

Security fixes are applied to the latest release series and to `master`.

| Version | Supported |
|---------|-----------|
| latest release | ✅ |
| older releases | ❌ (upgrade) |

## Reporting a Vulnerability

**Do not open a public issue for a suspected vulnerability.**

Report privately via GitHub's *Report a vulnerability* button on the
[Security advisories page](https://github.com/tpt-solutions/tpt-process/security/advisories),
or email `security@tpt-solutions.example` (replace `.example` with `.com`)
with:

- A description of the issue and its impact on computed results or
  deployments.
- Steps / a minimal reproducing case (a failing test is ideal).
- Any known workarounds.

You will receive an acknowledgement within 3 business days, and we aim to
release a fix within 90 days of confirmation. We credit reporters in the
advisory and `CHANGELOG.md` unless anonymity is requested.

## Scope

Of particular interest for engineering software:

- Numerical defects that produce **silently wrong physical results**
  (converged-but-wrong answers, unchecked overflow of physical quantities,
  incorrect equation constants).
- Memory-safety or UB issues (Rust `unsafe` is denied workspace-wide;
  any exception must be justified in writing and audited).
- Supply-chain issues in the dependency chain (see `deny.toml`; the license
  and advisory policy there is part of the security posture).

`wasm-bindgen` bindings in `tpt-proc-wasm` are in scope, including data
handling across the JS boundary.
