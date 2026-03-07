# 09 Agent/Actor Simulation Usage

## Why Agents (Not Only Aggregate Equations)

The model uses agent/actor simulation to make emergence explicit:

1. Individuals interact stochastically (cooperate, trade, conflict, migrate).
2. Local interactions shift trust, resources, inequality, and memory.
3. These shifts aggregate into society-level complexity and superorganism dynamics.

The model now includes an explicit actor-message loop in the evolutionary layer:

1. each society actor receives per-generation messages,
2. actor state is updated locally,
3. global aggregates are derived from actor states.

## Actor Levels

1. `MicroAgent`:
   - `resources`, `trust`, `status`, `aggression`, `cooperation`,
   - memory fields: `recent_conflict`, `recent_coop`,
   - `role`: functional role (`Producer` / `Coordinator` / `Trader`),
   - `affinity: [f64; 3]`: cultural identity vector for oxytocin in-group/out-group dynamics.
2. `AgentBasedSociety`:
   - collection of agents,
   - current subsistence mode,
   - coupling and ecological pressure,
   - topology (`ring`, `small-world`, `random`) and interaction radius,
   - demographic bounds (`min_population`, `max_population`),
   - interaction coefficients.
3. Macro projection:
   - `MicroMacroProjection` contract derived from agents each tick:
     - mean resources/trust/inequality,
     - event rates,
     - mode composition shares.
   - projected to `LocalSocietyState` (which now includes `GovernanceState`),
   - mapped into `local_complexity` and `emergence_from_projection`.
4. `GovernanceState` (per-society governance):
   - current policy (`Laissez` / `Redistributive` / `Extractive`),
   - tax rate and redistribution rate,
   - stress channel: `price_pressure` and `legitimacy` in [0, 1].
   - Updated each tick via `adapt_governance` based on surplus and ecological pressure.

## Per-Tick Execution Loop

1. Run micro interactions (`step_agent_based_society`):
   - probabilistic cooperation, conflict, trade, migration events,
   - role-based modifiers (coordinators boost cooperation, traders boost trade),
   - oxytocin modifiers based on affinity distance (bonding/othering),
   - affinity drift: cooperation → convergence, conflict → divergence, trade → mild convergence,
   - **raid resolution**: when conflict exceeds 25% of population and ecological stress is high, high-aggression agents form raiding parties that seize resources from low-aggression agents, with affinity polarization between raiders and victims.
2. Apply bounded demographic dynamics:
   - births, deaths, and replacement to enforce population floors/ceilings.
2. Convert micro state to macro proxy (`macro_from_agents`).
3. Compute complexity/emergence metrics with `micro_macro_projection`.
4. Apply regime transition logic (`next_subsistence_mode`).
5. Update ecological pressure and coupling feedbacks.

## Primary APIs

- Seed a society:
  - `seed_agent_based_society(count, mode, coupling, eco_pressure)`
  - `seed_agent_based_society_with_topology(...)`
- Step one tick:
  - `step_agent_based_society(&mut society)`
- Run full simulation:
  - `run_agent_based_simulation(society, ticks, transition_cfg)`
- Run multi-generation actor evolution:
  - `evolution::simulate_evolution(config)`

## Practical Workflow

1. Start from baseline seeds (`HunterGatherer` / `Sedentary` / `Agriculture`).
2. Sweep parameters:
   - interaction coefficients (cooperation/conflict/trade/migration),
   - topology and interaction radius,
   - coupling,
   - ecological pressure,
   - transition thresholds.
3. Compare outcomes by:
   - trajectory class,
   - peak/end superorganism,
   - complexity retention,
   - final H/S/A composition.

## Interactive First Step (TUI)

Use the terminal UI to watch agent interactions directly:

1. `make tui-life`
2. Observe per-tick changes in:
   - cooperation/conflict/trade/migration counts,
   - births/deaths/replacements,
   - trust and inequality,
   - mode and emergence indicators.

## What This Enables

- See emergence as a bottom-up process.
- Test whether local interactions are sufficient for large-scale behavior.
- Observe emergent tribal clustering from oxytocin-driven affinity dynamics.
- Track governance policy cycles (laissez-faire → redistributive → extractive → recovery).
- Observe inter-society wars and agent-level raids as emergent conflict resolution.

## War Mechanics (Two Layers)

### Society-Level Wars (`resolve_society_wars`)

Evaluated each tick in `run_emergence_simulation` between all society pairs:
- **War probability**: driven by governance stress (low legitimacy), ecological pressure, and surplus differential between societies. Network coupling acts as a peace dividend (reduces war probability).
- **Military strength**: `population × mode_multiplier × (1 + surplus) × governance_factor`. Extractive regimes mobilize +20% more; laissez-faire regimes -20%.
- **Resolution**: probabilistic, weighted by strength ratio. Winner gains population and surplus; both sides suffer casualties and ecological damage.
- **Legitimacy shock**: wars erode governance legitimacy (-4% for winners, -12% for losers).

### Agent-Level Raids (`step_agent_based_society`)

Triggered when conflict count exceeds 25% of agents and ecological stress is elevated:
- **Raid probability**: `0.15 × stress + 0.10 × conflict_rate`.
- **Mechanics**: top-20% by aggression form a raiding party; bottom-20% are victims. Raiders gain resources (+0.08 to +0.12) and status; victims lose resources and trust.
- **Affinity polarization**: raiders converge toward each other (forming a war band identity), victims converge toward each other (shared victimhood). This deepens in-group/out-group boundaries.
- Identify parameter zones producing:
  - stabilizing complexity,
  - overshoot/correction,
  - fragile transitions,
  - stagnant fragmentation.
4. `SocietyActor` (evolution layer):
   - continent-local state, population, complexity, surplus, trust, resilience,
   - mutable NK genome,
   - message-driven updates (`ClimateShock`, `ResourcePulse`, `MigrationLink`).
5. `WorldMap` (abstract geography layer):
   - configurable continental layouts (`Connected`, `Regional`, `Islands`),
   - tunable `isolation_factor` to test diffusion/isolation constraints.
6. `DunbarBehaviorModel` (social constraints):
   - configurable thresholds,
   - expectation load, trust decay, communication cost, coordination gain by scale.
