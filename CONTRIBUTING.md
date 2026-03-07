# Contributing

## Development Setup

```bash
uv sync --all-extras
```

## Quality Contract

All PRs must pass:

1. `uv run make lint`
2. `uv run make typecheck`
3. `uv run make test` (coverage threshold is enforced)

## Commit Frequency

Prefer small, regular commits aligned to one logical change.

## Modeling Changes

Any change to model assumptions must include:

1. a note in `docs/foundations/` or an assumption RFC,
2. tests for expected behavioral impact,
3. updated scenario metadata when relevant.
