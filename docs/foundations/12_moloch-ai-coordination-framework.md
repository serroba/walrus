# 12 Moloch, AI, and Coordination Failure Framework

## Why this matters for this project

This simulator is not only about emergence and collapse from biophysical limits.
It also needs to model **coordination failure dynamics** where individually rational actions produce globally harmful outcomes.

This document defines how to include those dynamics explicitly.

## Core Concepts to Represent

1. **Multipolar traps**
- multiple actors compete under partial trust and partial information,
- each actor is pressured to defect if others might defect,
- local optimization leads to global loss.

2. **Narrow objective optimization**
- actors optimize metrics such as growth, power, or short-term security,
- externalities are not fully priced into those objectives,
- system-level harm accumulates even while local metrics improve.

3. **Recursive acceleration via AI**
- capability improves optimization speed and strategic adaptation,
- faster optimization can amplify races to the bottom,
- governance latency becomes a major risk variable.

4. **Information integrity degradation**
- synthetic media and attention competition can reduce trust,
- lower trust reduces coordination capacity,
- lower coordination increases multipolar trap intensity.

5. **Nash equilibrium and equilibrium shifts**
- strategic choices should be represented as explicit games where possible,
- model should identify stable strategy profiles (local Nash-like attractors),
- interventions should be evaluated by whether they shift the system from lose-lose equilibria to win-win equilibria.

6. **Criticality and heavy-tail dynamics**
- systems near critical points can produce scale-free cascades,
- extreme events can dominate long-run outcomes,
- averages alone are insufficient for risk evaluation.

7. **Exogenous systemic shocks**
- natural disasters and pandemics must be represented explicitly,
- shocks can amplify coordination failures and destabilize equilibria,
- resilience should be assessed under repeated shock regimes, not only baseline dynamics.

## Modeling Additions (Conceptual)

1. Add strategic blocs and firms as explicit actors.
2. Add objective functions per actor:
- local utility,
- externality penalty (optional, policy dependent),
- strategic threat response.
3. Add treaty/coordination attempts with verification cost.
4. Add AI acceleration parameter affecting:
- innovation velocity,
- attack/defense dynamics,
- governance lag.
5. Add information environment state affecting trust and alignment.
6. Add explicit game layers:
- stage games (defect/cooperate, arms race, resource extraction),
- repeated games with memory and reputation,
- equilibrium diagnostics (best-response consistency, exploitability proxy).
7. Add criticality/tail layers:
- avalanche/cascade tracking (size, duration, inter-event times),
- power-law diagnostics for event distributions,
- branching-ratio proxy to detect near-critical regimes.
8. Add shock layers:
- disaster and pandemic event generators,
- health/infrastructure/production impacts,
- shock-to-governance and shock-to-trust couplings.

## Implemented: Trust-Memory Coordination Dilemma

The first concrete coupling between this framework and the agent simulation is now implemented via **trust-modulated cooperation with coordination failure measurement**.

### Mechanism

Each agent carries a `trust_memory` value in [0, 1] — an exponential moving average of cooperation received from neighbors:

`trust_memory_{t+1} = (1 - alpha) * trust_memory_t + alpha * observed_coop_rate`

where `alpha = trust_memory_decay` (default 0.15) and `observed_coop_rate` is the fraction of this tick's interactions that were cooperative.

Trust memory modulates cooperation tendency:

`coop_tendency += trust_memory * trust_coop_weight`
`conflict_tendency += (1 - trust_memory) * trust_coop_weight * 0.5`

This creates a genuine **coordination dilemma**: when trust is low, agents rationally choose conflict even when mutual cooperation would yield higher surplus. The resulting defection further erodes trust, producing the self-reinforcing Moloch dynamic.

### Coordination Failure Index

For each pairwise interaction, we compute:
- **actual surplus**: the resource delta from the chosen action (cooperation, conflict, or trade)
- **cooperative counterfactual**: the surplus that would result if that interaction were cooperative

The **coordination failure index** is:

`CFI_t = 1 - (sum_actual_surplus / sum_cooperative_optimal)`

Clamped to [0, 1]. CFI = 0 means all interactions achieve cooperative optimum. CFI = 1 means total coordination failure. This feeds into the superorganism index as a 9th component (weight 1.0), capturing the game-theoretic lock-in dimension.

### Reinforcing Loop (R8): Trust-Defection Spiral

`trust ↓ -> cooperation ↓ -> conflict ↑ -> trust ↓`

This is the micro-foundation for the Moloch dynamic. Under stress (ecological pressure, resource scarcity), conflict increases, trust erodes, and agents are locked into defection even when cooperation is Pareto-improving. The superorganism pathway emerges when hierarchical coercion (patron-client delegation) substitutes for voluntary cooperation.

### Falsifiability

The trust-memory model is wrong if:
- Populations with identical initial conditions but different initial trust levels converge to the same coordination failure index within 100 ticks (trust memory would be irrelevant).
- Sustained high cooperation does not raise mean trust (the EMA would be broken).
- Removing trust modulation (`trust_coop_weight = 0`) does not change coordination failure dynamics at all (the mechanism would be inert).

### Code Anchors

- Trust memory field: `Population.trust_memory` in `agents.rs`
- Cooperation modulation: `compute_interactions()` in `agents.rs`
- Trust EMA update: `apply_effects()` in `agents.rs`
- CFI computation: `measure_emergent_state()` in `agents.rs`
- Superorganism integration: `superorganism_index()` in `agents.rs`

### Configurable Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `trust_coop_weight` | 0.25 | How strongly trust_memory affects cooperation vs conflict tendency |
| `trust_memory_decay` | 0.15 | EMA decay rate (lower = longer memory, more path-dependent) |

## New State Variables to Track

### Implemented
- `coordination_failure_index` — fraction of surplus lost to non-cooperative interactions (0-1)
- `mean_trust` — population-level mean trust_memory (cooperation expectation)

### Future (conceptual)
- `arms_race_intensity`
- `externality_burden`
- `governance_capacity`
- `information_integrity`
- `catastrophe_risk_proxy`
- `nash_stability_score`
- `equilibrium_regime` (e.g., cooperative / mixed / competitive trap)
- `tail_exponent_alpha`
- `criticality_index`
- `cascade_size_p95`
- `extreme_event_share`

## Experimental Scenarios

1. Baseline competitive race.
2. Coordinated restraint with costly verification.
3. AI-accelerated race.
4. AI-accelerated race + governance intervention.
5. Isolation/fragmentation + synthetic media degradation.
6. Incentive redesign that changes payoff matrices and tests equilibrium regime transitions.
7. Suppression-vs-release policy tests to evaluate megacascade risk.

## Expected Research Value

This allows testing whether:
- collapse risk is mostly resource-driven, coordination-driven, or coupled,
- AI acceleration improves adaptation faster than it degrades coordination,
- policy levers can shift the attractor away from collapse/dystopia.
- superorganism emergence is a robust equilibrium across broad parameter regimes or a contingent one.
- tail risk is endogenous to game structure and coordination quality, not just random noise.

## Boundaries

1. This remains a hypothesis-testing system, not historical prediction.
2. Outputs must include uncertainty and maturity labels.
3. Avoid normative claims in reports; present mechanisms and tradeoffs.

## Superorganism Inevitability Hypothesis

Primary research question:

- Is superorganism emergence an inevitability under broad game-theoretic and biophysical constraints?

Operational criterion:

- If most parameterized game structures converge to high `superorganism_index` with high `nash_stability_score`, this supports inevitability.
- If emergence depends on narrow assumptions and breaks under plausible incentive redesign, inevitability is not supported.
