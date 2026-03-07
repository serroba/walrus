# Walrus Simulator

Open-source simulation framework to test explicit assumptions about energy, materials, institutions, and emergent superorganism dynamics.

## Tech Direction

- Engine core: Rust (performance, safety, scalability)
- Analysis/orchestration: optional Python layer later (via `uv`)

## Quality Gates

- Format: `cargo fmt --all -- --check`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Tests: `cargo test --workspace --all-targets`

## Quick Start

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```
