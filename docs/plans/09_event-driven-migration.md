# Plan 09: Migration to Event-Driven Simulation

## Goal

Replace the current tick-based batch simulation with a discrete-event simulation (DES)
to produce emergent, non-deterministic behavior. Agents should act on their own
stochastic timelines rather than in lockstep phases.

## Why not an actor framework

We evaluated Ractor, Kameo, Actix, Xtra, and Coerce. Actor frameworks solve service
architecture problems (fault tolerance, supervision, distribution) that don't apply here.
What we need — stochastic scheduling, emergent ordering, non-determinism — none of them
provide. We'd still have to build the event queue ourselves on top.

**Decision:** Use `tokio::mpsc` channels if async is needed later, but start with a
single-threaded priority-queue event loop. No framework dependency. Revisit if we need
distribution across machines (Ractor would be the pick).

---

## Current Architecture (what changes)

```
for tick in 0..N:
  1. build_spatial_grid()
  2. compute_interactions()    → parallel, returns InteractionEffects
  3. apply_effects()           → mutates Population
  4. energy_harvest_tick()     → mutates resources
  5. inter_society_tick()      → raids, conquest, tribute
  6. cultural_transmission()   → mutates cultures
  7. lifecycle_tick()          → aging, death, reproduction
  8. movement_tick()           → mutates positions
  9. measure_emergent_state()  → snapshot
```

All agents execute each phase simultaneously. Deterministic given a seed.

## Target Architecture

```
EventQueue (BinaryHeap, ordered by time)
  │
  ├── AgentEvent { time, agent_id, kind }
  │     kinds: Forage, Interact(neighbor), Move, Reproduce, Age, Learn, Transmit
  │
  ├── GroupEvent { time, kin_group, kind }
  │     kinds: Raid(target), CollectTribute, Migrate
  │
  └── WorldEvent { time, kind }
        kinds: RebuildSpatialIndex, MeasureState, DepletResources
```

Each event executes against the **current** world state (not a frozen snapshot),
applies its effects immediately, and schedules follow-up events with stochastic delays.

---

## Migration Phases

### Phase A: Event Loop Core (foundation)

**Files:** new `crates/walrus-engine/src/event_queue.rs`

Create the discrete-event simulation core:

```rust
pub struct Event {
    pub time: f64,
    pub kind: EventKind,
}

pub enum EventKind {
    Agent { id: u64, action: AgentAction },
    Group { kin_group: u32, action: GroupAction },
    World { action: WorldAction },
}

pub struct EventQueue {
    heap: BinaryHeap<Reverse<Event>>,  // min-heap by time
}
```

- `EventQueue::push(event)`, `EventQueue::pop() -> Option<Event>`
- `schedule_next(agent_id, action, rate, rng) -> Event` — time = now + exponential(1/rate)
- Main loop: `while let Some(event) = queue.pop() { dispatch(event, &mut world) }`
- World clock advances to each event's time (no fixed dt)

**Deliverable:** Event loop that can schedule and dispatch events. No simulation logic yet.

**Effort:** 1-2 days

---

### Phase B: Agent-Level Event Scheduling

**Files:** modify `agents.rs`, new `src/event_dispatch.rs`

Convert each phase into event types with per-agent stochastic rates:

| Current Phase | Event Type | Base Rate | Rate Modifiers |
|---|---|---|---|
| `compute_interactions` | `Interact(neighbor_id)` | ~1/tick | cooperation, aggression, proximity |
| `energy_harvest_tick` | `Forage` | ~1/tick | skill level, local EROEI |
| `cultural_transmission` | `Transmit` | ~0.3/tick | prestige of neighbors, trust |
| `lifecycle_tick` (aging) | `Age` | 1/tick | fixed biological clock |
| `lifecycle_tick` (reproduce) | `Reproduce` | fertility/tick | age, health, resources |
| `lifecycle_tick` (death) | `Die` | mortality/tick | age, health, starvation |
| `movement_tick` | `Move` | ~1/tick | drift magnitude, kin pull |

Key design decisions:
- **Interaction is two-sided:** when agent A schedules `Interact(B)`, the outcome
  affects both A and B immediately. B does not need to "respond" — this is not
  message passing, it's a world mutation.
- **Rates are state-dependent:** a starving agent forages more often, an aggressive
  agent initiates conflict more often. Rates recalculated when scheduling the *next*
  event of that type.
- **No global phases:** agent 7 might reproduce while agent 12 is mid-conflict.
  This is the source of non-determinism and emergence.

**Deliverable:** All agent-level actions converted to events. The old `simulate_agents`
tick loop replaced with the event loop.

**Effort:** 3-5 days

---

### Phase C: Spatial Index as a Lazy Service

**Files:** modify `SpatialGrid` in `agents.rs`

The spatial grid currently rebuilds every tick. In an event-driven model:

- **Option 1 (simple):** Rebuild periodically via a `WorldEvent::RebuildSpatialIndex`
  scheduled every N simulation-time units. Stale but fast.
- **Option 2 (incremental):** Update grid cells when agents move. Each `Move` event
  updates the agent's cell in O(1). Grid is always current.

**Recommendation:** Start with Option 1 (periodic rebuild at ~1 sim-time interval).
Switch to Option 2 only if staleness causes visible artifacts.

**Deliverable:** Spatial index works with event-driven model.

**Effort:** 1 day

---

### Phase D: Group-Level Events

**Files:** modify inter-society logic in `agents.rs`

Convert inter-society dynamics to group-level events:

- `Raid(attacker_group, target_group)` — scheduled when group mean aggression exceeds
  threshold. Rate proportional to aggression × group size.
- `CollectTribute(vassal, overlord)` — periodic event for existing tribute relations.
- `Migrate(agent_id, from_group, to_group)` — individual event triggered by low
  resources or low group prestige.

Group events aggregate member state at execution time (not pre-computed), so they
naturally reflect the current composition of the group.

**Deliverable:** Inter-society dynamics as events. Tribute relations managed via
recurring scheduled events.

**Effort:** 2-3 days

---

### Phase E: Measurement and Observation

**Files:** modify `measure_emergent_state`, example runner

Replace per-tick snapshots with time-windowed measurement:

- `WorldEvent::MeasureState` fires at regular intervals (e.g., every 1.0 sim-time)
- Snapshots capture population state at that moment, regardless of what events
  are in-flight
- Output format stays the same (CSV with time-series) but time column becomes
  continuous float instead of integer tick

Update `agent_simulation.rs` example to use new API:

```rust
let result = simulate_event_driven(cfg);
// result.snapshots now indexed by sim_time: f64
```

**Deliverable:** Observable output compatible with existing analysis.

**Effort:** 1 day

---

### Phase F: Remove Old Tick Loop

**Files:** `agents.rs`

- Delete `simulate_agents()` and the sequential phase functions
- Keep the computation logic (interaction math, cultural transmission rules, etc.)
  as pure functions called by event handlers
- Old tests that assert per-tick behavior get rewritten to assert statistical
  properties over time windows

**Deliverable:** Single simulation path, no dead code.

**Effort:** 1-2 days

---

## What Stays Unchanged

- **Population struct (SoA layout):** Still the most cache-efficient way to store
  agent state. Events mutate it directly.
- **Interaction math:** Cooperation/conflict/trade formulas stay identical.
- **Cultural transmission logic:** Same prestige-biased and peer adoption rules.
- **Lifecycle formulas:** Fertility curves, mortality, aging math.
- **Energy landscape:** EROEI curves and depletion.
- **AgentSimConfig:** All parameter structs stay. Add rate parameters for event
  scheduling.

## What Changes

| Component | Before | After |
|---|---|---|
| Time model | Discrete ticks (u32) | Continuous time (f64) |
| Execution order | Fixed phase sequence | Stochastic event ordering |
| Determinism | Fully deterministic (seeded) | Non-deterministic by design |
| Parallelism | Rayon par_iter per phase | Single-threaded event loop (for now) |
| Interaction timing | All agents interact simultaneously | Agents interact at their own pace |
| State consistency | Snapshot-then-apply | Immediate mutation on event |

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Performance regression (losing Rayon) | Slower for large populations | Profile first. Can batch nearby-time events for parallelism if needed |
| Runaway event storms | Agent A triggers B triggers A... | Cap event rate per agent. Minimum inter-event delay. |
| Stale spatial index | Wrong neighbors for interactions | Periodic rebuild frequency tuned to movement rate |
| Population mutation during events | Index invalidation on death | Use stable IDs (u64) not array indices. Generational arena or slot map for O(1) lookup |
| Hard to validate against old model | Can't diff outputs directly | Statistical validation: same parameter ranges should produce same macro-level distributions (Gini, hierarchy, conflict rates) |

## New Dependencies

- None required for Phase A-F (std `BinaryHeap` suffices)
- Optional: `slotmap` crate for stable agent handles (avoids index invalidation on death)

## Config Additions

```rust
pub struct EventParams {
    pub forage_base_rate: f64,      // mean events per sim-time unit
    pub interact_base_rate: f64,
    pub move_base_rate: f64,
    pub transmit_base_rate: f64,
    pub reproduce_base_rate: f64,
    pub raid_base_rate: f64,
    pub tribute_interval: f64,
    pub spatial_rebuild_interval: f64,
    pub measure_interval: f64,
}
```

## Estimated Total Effort

| Phase | Days | Depends On |
|---|---|---|
| A: Event loop core | 1-2 | — |
| B: Agent events | 3-5 | A |
| C: Spatial index | 1 | A |
| D: Group events | 2-3 | A, B |
| E: Measurement | 1 | A |
| F: Cleanup | 1-2 | B, C, D, E |
| **Total** | **9-14 days** | |

Phases A, C, and E can be developed in parallel once A's interface is defined.
B is the bulk of the work and the critical path.

## Testing Strategy

The migration introduces two fundamentally different categories of behavior:
deterministic mechanics (the math inside each event) and emergent dynamics (what
comes out of the interactions). These require different testing approaches.

### Layer 1: Unit Tests — Deterministic Mechanics

The pure computation functions (interaction math, cultural transmission rules,
fertility curves, EROEI calculations) are unchanged and remain fully deterministic.
Test them in isolation, outside the event loop.

```rust
// These stay as normal #[test] assertions
#[test]
fn cooperation_payoff_scales_with_skill_complement() { ... }

#[test]
fn conflict_winner_determined_by_power_score() { ... }

#[test]
fn fertility_peaks_at_configured_age() { ... }

#[test]
fn eroei_declines_with_depletion() { ... }
```

**Principle:** Every formula that computes a result from inputs gets a unit test.
The event system is just scheduling — the math must be correct regardless of when
it runs.

### Layer 2: Unit Tests — Event Queue Mechanics

The event loop infrastructure is deterministic and testable:

```rust
#[test]
fn events_dispatch_in_time_order() { ... }

#[test]
fn scheduling_with_rate_produces_exponential_intervals() { ... }

#[test]
fn dead_agent_events_are_skipped() { ... }

#[test]
fn event_storm_cap_limits_per_agent_rate() { ... }
```

### Layer 3: Property-Based Tests — Invariants That Must Always Hold

Use `proptest` or `quickcheck` to verify structural invariants across randomized
runs. These don't assert specific outcomes — they assert that the simulation never
breaks its own rules.

```rust
// Run N events with random seed, then check:
#[test]
fn resources_never_go_negative() { ... }

#[test]
fn dead_agents_generate_no_further_events() { ... }

#[test]
fn population_bounded_by_config_max() { ... }

#[test]
fn all_agents_belong_to_a_valid_kin_group() { ... }

#[test]
fn spatial_positions_stay_within_world_bounds() { ... }

#[test]
fn tribute_relations_reference_existing_groups() { ... }
```

**Principle:** The simulation is non-deterministic, but it must be *valid*. Invariants
are the contract between "anything can happen" and "nonsense doesn't happen."

### Layer 4: Statistical Tests — Emergent Behavior Validation

These tests run many simulations and assert on distributions, not individual outcomes.
They answer: "does the system produce plausible macro-level patterns?"

```rust
#[test]
fn gini_coefficient_increases_with_hierarchy() {
    // Run 50 sims with high-authority culture params
    // Run 50 sims with low-authority culture params
    // Assert: mean Gini of high-authority > mean Gini of low-authority
    // Use Welch's t-test or Mann-Whitney U, p < 0.05
}

#[test]
fn conflict_rate_correlates_with_aggression() {
    // Sweep aggression parameter across range
    // Assert: Spearman rank correlation > 0.7
}

#[test]
fn population_reaches_carrying_capacity() {
    // Run 30 sims to steady state
    // Assert: mean final population within 20% of expected carrying capacity
}

#[test]
fn cultural_homogeneity_increases_over_time() {
    // Run 30 sims for extended time
    // Assert: mean cultural variance at t=end < mean cultural variance at t=start
    // (prestige-biased transmission should cause convergence)
}
```

**Principle:** Don't test "what happened" — test "what kind of thing tends to happen."
Use effect sizes and confidence intervals, not exact value equality.

**Implementation notes:**
- These tests are slow. Gate them behind `#[ignore]` or a `--features statistical-tests`
  flag. Run in CI nightly, not on every push.
- Use a fixed sample size large enough for statistical power (30-50 runs minimum per
  condition for t-tests).
- Log seeds for failed runs so they can be investigated individually.
- Accept that ~5% of runs will fall outside confidence intervals by chance. Use
  Bonferroni correction if running many statistical tests.

### Layer 5: Regression Baselines — Catch Unintended Drift

Before removing the old tick-based model (Phase F), run both engines with equivalent
parameters and record macro-level distributions:

- Population size over time
- Gini coefficient distribution
- Conflict rate / cooperation rate
- Mean kin group size
- Cultural diversity index

Store these as baseline distributions. After migration, the event-driven model should
produce statistically similar distributions (KS test, p > 0.05) for the same parameter
ranges. Not identical runs — similar *kinds* of outcomes.

**This is a one-time validation gate before deleting the old code.**

### Test Pyramid Summary

```
         ╱ Statistical (slow, nightly) ╲
        ╱   30-50 runs per condition     ╲
       ╱   assert on distributions        ╲
      ╱─────────────────────────────────────╲
     ╱  Property-based (medium, every PR)    ╲
    ╱   randomized inputs, assert invariants  ╲
   ╱───────────────────────────────────────────╲
  ╱  Unit tests (fast, every commit)            ╲
 ╱   deterministic math + event queue mechanics  ╲
╱─────────────────────────────────────────────────╲
```

---

## Open Questions

1. **Reproducibility:** Do we want an optional deterministic mode (fixed RNG,
   deterministic event ordering as tiebreaker)? Useful for debugging.
2. **Performance floor:** What's the minimum acceptable population size? If we need
   100k+ agents, we may need batched event processing.
3. **Observation frequency:** How often should MeasureState fire? Too often adds
   overhead; too rarely misses transient dynamics.
4. **Agent identity:** Switch from array index to `slotmap::SlotMap` now, or keep
   indices with a free-list?
