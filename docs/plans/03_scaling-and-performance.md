# Plan 03: Scaling and Performance

## Goal

Scale architecture from laptop to cluster while preserving deterministic science workflows.

## Strategy

1. Data-oriented memory layouts (SoA over AoS).
2. Parallel step execution with deterministic reduction phases.
3. Partitioning by geography/network communities.
4. Optional distributed backend for very large runs.

## Performance Workstreams

1. Profiling harness and flamegraph automation.
2. Hot-path optimization budget process.
3. Parallel RNG stream management.
4. Storage optimization (columnar snapshots, checkpoint cadence).

## Acceptance Criteria

- 10M+ agent runs on workstation tier.
- Stable scaling efficiency across cores.
- Consistent outputs across single-thread and multi-thread deterministic modes.
