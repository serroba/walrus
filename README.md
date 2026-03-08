# Walrus Simulator

Open-source agent-based simulation framework testing whether **superorganism emergence is inevitable** given enough time and resources. Individual agents interact locally (cooperate, trade, conflict, delegate, court) and macro patterns like hierarchy, specialization, inequality, and coordinated throughput-maximizing behavior emerge from these micro interactions.

## Tech Stack

- Engine: Rust (struct-of-arrays layout, rayon parallelism, spatial hashing)
- Analysis/orchestration: optional Python layer later (via `uv`)

## Architecture

### Agent simulation (`agents.rs` — primary)

Individual agents with traits, organized in a struct-of-arrays layout for cache efficiency:

- **Phase 1 — Individual agents**: sex, age, fertility, health, skills (forager/crafter/builder/leader/warrior), status, prestige, aggression, cooperation, resources. Interactions: cooperation, trade, conflict, delegation, courtship. Lifecycle: aging, death, reproduction with trait inheritance.
- **Phase 2 — Energy model**: EROEI dynamics with 4 energy types (biomass, agriculture, fossil, renewable). Tech-gated transitions via mean agent innovation. Spatial energy landscape with per-cell sources, depletion, and regeneration.
- **Phase 3 — Emergent institutions**: detected (not hardcoded) from population state — band/tribe/chiefdom/state classification from hierarchy depth, specialization, coercion rate. Patron-client hierarchies with inheritance. Public goods investment.
- **Phase 4 — Inter-society interactions**: kin-group level raids (power-based with aggression threshold), conquest triggering tribute relations, per-tick tribute collection, resource-stress migration between groups.
- **Phase 5 — Cultural transmission**: rich Culture struct (kinship system, marriage rule, residence rule, inheritance rule + authority, coercion tolerance, sharing, property, trust, risk norms + techniques bitfield). Three transmission mechanisms: vertical (parent→child), horizontal (peer→peer), oblique (prestige-biased).
- **Phase 6 — Superorganism detection**: composite index from 8 weighted components (hierarchy, inequality, specialization, institutional type, coercion, energy throughput, cultural authority, tribute). Convergence experiment sweeping 8 conditions across N seeds.

### Legacy layers

- `lib.rs`: stock-flow macro model, governance module, oxytocin affinity model
- `evolution.rs`: society-level actor model with NK fitness, Dunbar constraints, continent topology
- `calibration.rs`: OWID/Maddison calibration objectives
- `ensemble.rs`: uncertainty bands and robustness summaries

## Quick Start

```bash
# Check everything compiles and passes
make check

# Or step by step:
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

## Running the Agent Simulation

### Single run with CSV output

```bash
make agent-sim
```

Runs 500 ticks with 150 agents. Outputs CSV to stdout with 42 columns covering population, resources, inequality, energy, institutions, inter-society events, and cultural metrics. All parameters are env-configurable:

```bash
INITIAL_POP=300 WORLD_SIZE=50 TICKS=1000 make agent-sim > output.csv
```

### Superorganism convergence experiment

```bash
make agent-convergence
```

Tests the core hypothesis across 8 starting conditions (baseline, high density, rich/scarce energy, aggressive, cooperative, hierarchical, island) with multiple seeds:

```bash
SEEDS=50 TICKS=2000 make agent-convergence > results.csv
```

Outputs per-condition CSV with arrival rates, peak/final superorganism index, institutional/kinship/marriage distributions. Also prints key questions analysis to stderr: hierarchy→collapse correlation, kinship→energy coupling, fossil→superorganism relationship.

### Key environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `INITIAL_POP` | 150 | Starting population |
| `TICKS` | 500 | Simulation length |
| `WORLD_SIZE` | 100 | World dimensions |
| `INTERACTION_RADIUS` | 8 | Neighbor search radius |
| `SEEDS` | 10 | Seeds per condition (convergence) |
| `THRESHOLD` | 0.35 | Superorganism threshold (convergence) |
| `SUSTAINED` | 20 | Ticks above threshold to count (convergence) |

See `examples/agent_simulation.rs` for the full list of 80+ configurable parameters.

## Other Examples

| Command | Description |
|---------|-------------|
| `make tui-life` | Live terminal agent simulation |
| `make convergence-experiment` | Legacy evolution-based convergence |
| `make evolution-run` | Multi-generation actor evolution |
| `make evolution-sweep` | Isolation level comparison |
| `make viz-report` | Generate calibration report |
| `make viz-app` | Interactive dashboard |
| `make system-feedback` | System feedback tests + emergence run |

## Quality Gates

```bash
make check              # fmt + clippy + tests
make feedback-loop      # check + coverage (90% engine)
```

## Plan

The full 6-phase implementation plan is in [`docs/plans/08_emergent-society-evolution.md`](docs/plans/08_emergent-society-evolution.md). All phases are complete.

## Documentation

- `docs/foundations/` — explicit assumptions, mathematical model, simulation design
- `docs/plans/` — phased implementation plans
- `docs/foundations/09_agent-actor-simulation.md` — agent/actor usage guidance
- `docs/foundations/12_moloch-ai-coordination-framework.md` — coordination-failure and AI-risk framing
