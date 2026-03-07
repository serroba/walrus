# Plan 01: Project Bootstrap

## Goal

Create a production-grade open-source base for a simulation engine.

## Deliverables

1. Repository structure:
   - `engine/`, `models/`, `scenarios/`, `experiments/`, `docs/`, `benchmarks/`.
2. CI pipeline:
   - lint, unit tests, reproducibility smoke test.
3. Scenario manifest format (YAML/TOML + schema validation).
4. Deterministic random seed system.
5. Data output contract (Parquet/Arrow + metadata).

## Acceptance Criteria

- Reproducible run from single command.
- Seeded runs produce byte-identical summary outputs.
- New model module can be added via plugin interface.
