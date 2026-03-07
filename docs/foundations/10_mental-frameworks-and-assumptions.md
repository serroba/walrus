# 10 Mental Frameworks and Assumptions

## Purpose

This project is a dynamics laboratory, not a deterministic replay of history.
The objective is to test classes of behavior:

- emergence of complexity,
- lock-in and coordination gains,
- overshoot and local collapse,
- reorganization under constraints.

## Framework Stack

1. **Dunbar numbers (social scaling):**
- social structure changes as population scales through layers (`5, 15, 50, 150, 500, 1500`).
- model impact: scaling is modeled as a behavior profile, not only hard thresholds:
  - expectation load rises with scale,
  - trust decay pressure rises with scale,
  - communication costs rise with scale,
  - coordination gains also rise with scale.
- thresholds are configurable to test alternative social scaling assumptions.

2. **Jared Diamond-style geography constraints:**
- complexity growth depends on geography and biophysical opportunity, not culture alone.
- model variables: domesticable biomass, diffusion access, energy endowment, carrying capacity.
- topology controls: abstract layouts (`connected`, `regional`, `islands`) and `isolation_factor`.
- interpretation: continents with higher energy/access should show earlier complexity emergence.

3. **Tainter-style diminishing returns to complexity:**
- complexity initially solves coordination problems, then maintenance burden rises nonlinearly.
- model variable: maintenance includes a convex complexity term.
- expected dynamic: growth -> lock-in -> fragility -> collapse under stress/depletion.

4. **NK adaptive landscape (Kauffman):**
- institutional/behavioral adaptation is rugged and path-dependent.
- model variable: each society has a binary genome evaluated by NK fitness.
- mutation + selection explores local adaptive peaks over generations.

5. **Actor-model interaction loop:**
- societies are explicit actors receiving messages each generation.
- message classes: climate shock, resource pulse, migration link, natural disaster, pandemic wave.
- actor state update is local and then aggregated globally.

6. **Energy/material accounting + ecological feedback:**
- extraction increases surplus but depletes local stocks.
- low stock/high depletion triggers collapse pressure.
- regeneration prevents trivial one-way decline and enables reorganization cycles.

7. **Exogenous shocks (disasters and pandemics):**
- natural disasters and pandemics are modeled as recurrent, stochastic shocks,
- shock risk interacts with geography, connectivity, and local stress,
- shocks can trigger non-linear collapse/regression pathways.

## What We Assume Explicitly

1. Local surplus and complexity are coupled but not monotonic.
2. Collapse can be local, partial, and reversible.
3. Migration/trade corridors can diffuse complexity and shocks.
4. Isolation constraints change whether adaptation converges globally or diverges locally.
5. Different geographies produce different timing and depth of complexity transitions.
6. Uncertainty is intrinsic: we report ensembles and confidence labels.

## What We Do Not Claim

1. Exact historical replay for any specific civilization.
2. Normative claims about what societies should do.
3. Single-cause collapse narratives.

## Code Anchors

- Evolutionary actor map simulation: `crates/walrus-engine/src/evolution.rs`
- Agent/actor micro simulation: `crates/walrus-engine/src/lib.rs`
- Calibration and confidence: `crates/walrus-engine/src/calibration.rs`
- Ensemble uncertainty: `crates/walrus-engine/src/ensemble.rs`
