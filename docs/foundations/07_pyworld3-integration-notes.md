# 07 PyWorld3 Integration Notes

Reference examined: [serroba/pyworld3](https://github.com/serroba/pyworld3)

## Why It Helps

PyWorld3 provides a strong macro foundation for:

1. Sector stock-flow accounting (population, capital, agriculture, pollution, resources).
2. Explicit delays (information and material delays).
3. Cross-sector coupling that can generate overshoot/collapse dynamics.

These are directly relevant for superorganism constraints and collapse pressure.

## What It Does Not Cover (for our objective)

1. Agent heterogeneity and social network effects.
2. Group-size-dependent behavioral transitions.
3. Emergence of local complex societies as separate interacting units.

## Hybrid Modeling Decision

Use a two-layer model:

1. `Local ABM layer`:
   - many local societies (bands, villages, city-polities),
   - explicit group-size and subsistence transitions,
   - local emergence metrics (hierarchy, lock-in, specialization, coercion).
2. `Macro SD layer` (World3-inspired):
   - global/regional stocks and delays,
   - energy/material/ecology constraints,
   - balancing loops that limit or destabilize growth.

## Mapping to Our Current Model

- World3 population/capital/agriculture/pollution/resources -> macro constraint state.
- Our `group_behavior_profile` + `emergent_dynamics` -> local emergence state.
- Our `emergence_order_parameters` -> bridge to global superorganism index.

## Immediate Implementation Guidance

1. Represent world as many local societies with distinct `N, mode, surplus, coupling, eco pressure`.
2. Compute local complexity index per society each tick.
3. Aggregate local indices into global order parameters.
4. Add cross-society coupling (trade, migration, conflict) with delay.
5. Validate that complex societies emerge first locally, then synchronize into macro superorganism dynamics.
