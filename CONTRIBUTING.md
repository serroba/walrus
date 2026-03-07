# Contributing

## Development Setup

Install stable Rust (toolchain with `rustfmt` and `clippy`).

## Quality Contract

All PRs must pass:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace --all-targets`
4. Coverage gate (core engine): `cargo llvm-cov --package walrus-engine --all-targets --fail-under-lines 90 --summary-only`
5. Coverage gate (workspace): `cargo llvm-cov --workspace --all-targets --fail-under-lines 80 --summary-only`

## Coverage Policy

- We enforce per-scope thresholds in CI:
  - `walrus-engine` (core model logic): **90%** line coverage minimum.
  - workspace aggregate: **80%** line coverage minimum.
- Exclusions should be rare and justified (for generated code, glue code, or infrastructure wrappers with low test value).
- Any change to core model behavior must include or update tests.

## Developer Feedback Loop

Use fast, repeated quality loops while implementing:

1. `make check` after each logical change.
2. `make coverage-engine` before commit for model-core edits.
3. `make feedback-loop` before push.

## Commit Frequency

Prefer small, regular commits aligned to one logical change.

## Modeling Changes

Any change to model assumptions must include:

1. a note in `docs/foundations/` or an assumption RFC,
2. tests for expected behavioral impact,
3. updated scenario metadata when relevant.
