# 07 Calibration-First Validation Plan

## Goal

Move the simulator from demonstration to meaningful validation by anchoring to external benchmark series and reporting uncertainty explicitly.

## Scope

1. Agent-based core with explicit emergence mechanics:
- topology-driven partner selection (`ring`, `small-world`, `random`),
- probabilistic events (cooperate/trade/conflict/migrate),
- demographic turnover (birth/death/replacement),
- memory (`recent_conflict`, `recent_coop`).

2. Calibration layer with OWID+Maddison-compatible ingestion:
- canonical benchmark series: population, urbanization proxy, GDP/capita proxy, energy proxy,
- parameter vector bounds for interaction + transition coefficients,
- objective based on stylized direction and turning-point windows.

3. Ensemble uncertainty:
- N-seed and perturbed-parameter sweeps,
- median and p10/p90 trajectories,
- robustness score and maturity label.

4. Communication surfaces:
- TUI for visible micro emergence,
- standalone viewer with uncertainty bands and event annotations,
- report including fit diagnostics and robustness.

## Acceptance Gates

1. Core behavior:
- fixed-seed determinism under fixed topology,
- bounded event rates/resources/trust,
- hysteresis verified through stress/recovery tests.

2. Calibration:
- schema checks for benchmark adapters,
- optimizer smoke test improves objective vs baseline,
- stylized targets derived and compared.

3. Ensemble:
- percentile monotonicity (`p10 <= p50 <= p90`),
- robustness score in `[0,1]` and surfaced in report/viewer.

4. Milestone:
- confidence state at least `calibrated-stylized`,
- uncertainty shown in viewer,
- report exposes benchmark fit diagnostics,
- TUI shows non-flat regime shifts.
