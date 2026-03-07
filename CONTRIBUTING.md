# Contributing

## Development Setup

Install stable Rust (toolchain with `rustfmt` and `clippy`).

## Quality Contract

All PRs must pass:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace --all-targets`
4. Coverage gate: `cargo llvm-cov --workspace --all-targets --fail-under-lines 85 --summary-only`

## Coverage Policy

- We enforce a minimum **85% line coverage** in CI.
- Exclusions should be rare and justified (for generated code, glue code, or infrastructure wrappers with low test value).
- Any change to core model behavior must include or update tests.

## Commit Frequency

Prefer small, regular commits aligned to one logical change.

## Modeling Changes

Any change to model assumptions must include:

1. a note in `docs/foundations/` or an assumption RFC,
2. tests for expected behavioral impact,
3. updated scenario metadata when relevant.
