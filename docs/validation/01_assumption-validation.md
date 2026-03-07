# Assumption Validation (Current State)

## Scope of This Validation

This review checks whether documented assumptions in the foundations are reflected in executable model behavior.

Checked artifacts:

- `docs/foundations/02_mathematical-foundations.md`
- `docs/foundations/06_emergence-rules-and-feedbacks.md`
- `docs/foundations/07_pyworld3-integration-notes.md`
- `docs/foundations/08_visualization-guidelines.md`
- `crates/walrus-engine/src/lib.rs`
- visualization outputs in `outputs/latest/` and `docs/assets/`

## Validation Summary

The project currently validates **directional emergence behavior** (group size and mode shifts) but does **not yet validate historical realism** for long-term civilizational trajectories.

Status:

1. Directional model consistency: **Good**
2. Internal test/quality discipline: **Good**
3. Empirical/historical calibration: **Missing**
4. Macro stock-flow integration (World3 layer): **Missing**
5. Game-theoretic equilibrium diagnostics (Nash/Moloch): **Missing**
6. Superorganism inevitability testing protocol: **Missing**
7. Criticality and heavy-tail diagnostics: **Missing**
8. Explicit disaster/pandemic stress modeling in coordination layer: **Partially implemented**

## Assumption-to-Implementation Matrix

1. Group-size increases hierarchy/coordination pressure:
   - Status: **Implemented and tested**
   - Evidence: `group_behavior_profile`, tests for monotonic hierarchy increase.
2. Subsistence transitions change behavior (HG -> Sedentary -> Agriculture):
   - Status: **Implemented and tested**
   - Evidence: `emergent_dynamics`, `next_subsistence_mode`, transition tests.
3. Reinforcing loops produce superorganism tendencies:
   - Status: **Implemented (stylized), partially validated**
   - Evidence: `emergence_order_parameters`, scenario sweeps and system-feedback tests.
4. Ecological pressure acts as balancing loop:
   - Status: **Implemented and tested**
   - Evidence: ecological-pressure sensitivity tests and stress scenarios.
5. Local societies emerge first, then aggregate globally:
   - Status: **Implemented and tested**
   - Evidence: `local_complexity`, `aggregate_from_local_societies`, multi-society simulation.
6. World3-style macro constraints/delays shape long-run outcomes:
   - Status: **Not implemented**
   - Gap: currently no explicit capital, pollution, resource sector stock-flow layer.
7. Historical trajectory plausibility across millennia:
   - Status: **Not validated**
   - Gap: no calibration against historical data, no uncertainty envelope.
8. Explicit game-theoretic equilibrium structure and diagnostics:
   - Status: **Not implemented**
   - Gap: no payoff matrix abstractions, no repeated-game equilibrium classification, no Nash stability score.
9. Superorganism inevitability hypothesis testing:
   - Status: **Not implemented**
   - Gap: no formal criteria for inevitability vs contingency across parameter/game sweeps.
10. Heavy-tail and criticality modeling:
   - Status: **Not implemented**
   - Gap: no cascade event logs, no power-law/log-normal tail diagnostics, no criticality index.
11. Disaster/pandemic systemic stress:
   - Status: **Partially implemented**
   - Evidence: actor messages now include natural disaster and pandemic shock events.
   - Gap: no dedicated tail-risk diagnostics for shock cascades; no policy-response module yet.

## Why This Is Not Yet "Meaningful Enough"

Current model is a strong **hypothesis engine**, not yet a historical explanatory model.

Major blockers:

1. No empirical calibration target (historical proxies for complexity, urbanization, governance centralization, ecological stress).
2. No uncertainty quantification (single deterministic trajectories dominate interpretation).
3. No macro sector feedback layer (resource depletion/pollution/population-capital interactions are simplified).
4. No migration/conflict/trade delay mechanisms between local societies.

## Visualization Review

Current visuals are useful but still early-stage for general audiences.

Strengths:

1. Clear side-by-side trend lines for superorganism vs complexity.
2. Scenario behavior labels (stabilizing/fragile/overshoot/stagnant).
3. Final social composition bars (H/S/A).

Remaining clarity gaps:

1. No uncertainty bands or ensemble ranges.
2. No explicit event annotations (e.g., transition or regression events).
3. No plain-language panel explaining "what changed this curve" in each scenario.
4. No confidence indicator about model maturity.

## Priority Next Steps

1. Add calibration targets and benchmark scenarios (historical stylized facts with acceptable error ranges).
2. Add Monte Carlo/ensemble runs and visualize percentile bands.
3. Implement first macro stock-flow module (resource + pollution + productivity delay).
4. Add event markers to timelines (mode transitions, ecological threshold crossings).
5. Add a public-facing "Model Maturity" panel in the standalone viewer.
6. Implement game-theory layer with payoff structures and equilibrium metrics.
7. Add inevitability report: fraction of runs converging to stable superorganism equilibria under varied incentive regimes.
8. Add criticality report: tail exponents, extreme-event share, and suppression-vs-release comparisons.

## Bottom Line

The model is now technically solid for iterative exploration and parameter tuning.
It is **not yet sufficient** for strong claims about real historical causality.
