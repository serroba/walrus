# 09 Agent/Actor Simulation Usage

## Why Agents (Not Only Aggregate Equations)

The model uses agent/actor simulation to make emergence explicit:

1. Individuals interact locally (cooperate, trade, conflict).
2. Local interactions shift trust, resources, and inequality.
3. These shifts aggregate into society-level complexity and superorganism dynamics.

## Actor Levels

1. `MicroAgent`:
   - `resources`, `trust`, `status`, `aggression`, `cooperation`.
2. `AgentBasedSociety`:
   - collection of agents,
   - current subsistence mode,
   - coupling and ecological pressure.
3. Macro projection:
   - `LocalSocietyState` derived from agents each tick,
   - mapped into `local_complexity` and `emergence_order_parameters`.

## Per-Tick Execution Loop

1. Run micro interactions (`step_agent_based_society`):
   - cooperation, conflict, trade events.
2. Convert micro state to macro proxy (`macro_from_agents`).
3. Compute complexity/emergence metrics.
4. Apply regime transition logic (`next_subsistence_mode`).
5. Update ecological pressure and coupling feedbacks.

## Primary APIs

- Seed a society:
  - `seed_agent_based_society(count, mode, coupling, eco_pressure)`
- Step one tick:
  - `step_agent_based_society(&mut society)`
- Run full simulation:
  - `run_agent_based_simulation(society, ticks, transition_cfg)`

## Practical Workflow

1. Start from baseline seeds (`HunterGatherer` / `Sedentary` / `Agriculture`).
2. Sweep parameters:
   - interaction traits (trust/aggression/cooperation),
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
   - cooperation/conflict/trade counts,
   - trust and inequality,
   - mode and emergence indicators.

## What This Enables

- See emergence as a bottom-up process.
- Test whether local interactions are sufficient for large-scale behavior.
- Identify parameter zones producing:
  - stabilizing complexity,
  - overshoot/correction,
  - fragile transitions,
  - stagnant fragmentation.
