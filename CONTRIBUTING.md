# Contributing

## Development Setup

Install stable Rust (toolchain with `rustfmt` and `clippy`).

## Quality Contract

All PRs must pass:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace --all-targets`

## Commit Frequency

Prefer small, regular commits aligned to one logical change.

## Modeling Changes

Any change to model assumptions must include:

1. a note in `docs/foundations/` or an assumption RFC,
2. tests for expected behavioral impact,
3. updated scenario metadata when relevant.
