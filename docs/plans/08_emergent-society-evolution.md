# 08 Emergent Society Evolution

## Problem Statement

The current simulation is a systems dynamics model with extra steps. Societies
are bundles of floats updated by coupled equations. "Civilization" is a
threshold on a number. Dunbar transitions are table lookups. No agent decides
anything. No institution is invented. No trade or war happens between societies.
Energy is an undifferentiated scalar. Social structure (gender, kinship, rank)
doesn't exist.

To test whether superorganism emergence is inevitable, we need to observe it
emerging from individual-level interactions rather than computing it from
formulas. This plan describes how to get there.

## Design Principles

1. **Emergence over prescription** -- macro patterns must arise from micro
   rules, not be hardcoded as thresholds or formulae.
2. **Minimal viable agents** -- each agent carries only the traits needed for
   the dynamics we want to observe. No trait without a mechanism that uses it.
3. **Incremental validation** -- each phase must produce testable predictions
   before the next phase begins. No "build everything then see."
4. **Performance by design** -- struct-of-arrays layout, rayon parallelism,
   and spatial partitioning from day one. Target: 1M+ agents on a workstation.
5. **Existing code as scaffolding** -- keep the current macro emergence
   metrics as *diagnostic instruments* applied to the new micro layer, not as
   the simulation itself.

---

## Phase 1: Individual Agents with Traits ✅ COMPLETE

**Goal:** Replace `SocietyActor` (a society-level float bundle) with a
population of individual agents whose interactions produce group-level
behavior.

**Status:** Implemented in `crates/walrus-engine/src/agents.rs` (~1400 lines).
All parameters are configurable via `AgentSimConfig` sub-structs
(`InteractionParams`, `LifecycleParams`, `MovementParams`, `MateSelectionParams`).
Example runner in `examples/agent_simulation.rs` supports env var overrides for
every parameter. Run with `make agent-sim`.

### 1.1 Agent Model

```
struct Agent {
    id: u64,
    // Biology
    sex: Sex,               // Male | Female
    age: u16,               // generations lived
    fertility: f32,         // reproductive capacity (declines with age)
    health: f32,            // 0-1, affects productivity and survival

    // Skills and production
    skill_type: SkillType,  // Forager | Crafter | Builder | Leader | Warrior
    skill_level: f32,       // proficiency in current skill

    // Social
    status: f32,            // perceived rank within group
    prestige: f32,          // accumulated reputation (slow-changing)
    aggression: f32,        // tendency toward conflict
    cooperation: f32,       // tendency toward sharing/helping

    // Resources
    resources: f32,         // personal resource stock
    surplus: f32,           // excess beyond immediate needs

    // Cultural
    norms: u64,             // bitfield: cultural traits (analogous to NK genome)
    innovation: f32,        // propensity to adopt/create new techniques

    // Relationships
    kin_group: u32,         // kinship cluster ID
    partner: Option<u64>,   // mate (if paired)
    patron: Option<u64>,    // leader/patron they follow
}

enum Sex { Male, Female }

enum SkillType {
    Forager,    // hunting/gathering
    Crafter,    // tool/clothing/pottery making
    Builder,    // shelter/infrastructure
    Leader,     // coordination/dispute resolution
    Warrior,    // defense/raiding
}
```

### 1.2 Agent Interactions (per tick)

Each tick, agents interact with neighbors based on topology:

1. **Cooperation** -- share resources with kin/allies. Builds trust.
2. **Trade** -- exchange resources based on complementary skills. Builds surplus.
3. **Conflict** -- compete for resources/status. Redistributes or destroys.
4. **Courtship** -- mate selection based on status, resources, prestige.
   This is the sexual selection mechanism at the individual level.
5. **Teaching** -- transfer skill/norms to younger agents or offspring.
6. **Delegation** -- high-status agents receive coordination requests.
   This is how hierarchy *emerges* rather than being assigned.

### 1.3 Lifecycle

- **Birth**: offspring inherits kin_group, partial norms from both parents,
  skill_type influenced by parents but with mutation.
- **Maturation**: skill_level grows through practice and teaching.
- **Aging**: health and fertility decline. Prestige may increase.
- **Death**: from old age, low health, conflict, or resource starvation.

### 1.4 Data Layout (SoA for performance)

```
struct Population {
    // Parallel arrays for cache-friendly access
    ids: Vec<u64>,
    sexes: Vec<Sex>,
    ages: Vec<u16>,
    fertilities: Vec<f32>,
    healths: Vec<f32>,
    skill_types: Vec<SkillType>,
    skill_levels: Vec<f32>,
    statuses: Vec<f32>,
    prestiges: Vec<f32>,
    aggressions: Vec<f32>,
    cooperations: Vec<f32>,
    resources: Vec<f32>,
    surpluses: Vec<f32>,
    norms: Vec<u64>,
    innovations: Vec<f32>,
    kin_groups: Vec<u32>,
    partners: Vec<Option<u64>>,
    patrons: Vec<Option<u64>>,
    // Spatial index for neighbor lookups
    locations: Vec<(f32, f32)>,
}
```

This layout enables rayon `par_chunks_mut` for parallel agent updates and
SIMD-friendly memory access patterns. At ~100 bytes per agent, 1M agents =
~100MB, 10M = ~1GB.

### 1.5 What Emerges (not coded, observed)

- **Dunbar-like grouping**: agents naturally cluster around kin_group and
  patron relationships. Group sizes emerge from interaction range and
  cooperation/conflict balance.
- **Skill specialization**: when trade is profitable, agents who specialize
  outperform generalists, driving labor division.
- **Status hierarchy**: agents who coordinate successfully accumulate prestige.
  Others delegate to them. This is proto-hierarchy.
- **Inequality**: resource accumulation differences emerge from skill, luck,
  trade position, and inheritance.

### 1.6 Validation

- Run with 150 agents in a tight topology. Observe whether stable groups of
  ~5, ~15, ~50 form through interaction patterns alone.
- Measure Gini coefficient over time. Should rise as specialization increases.
- Track patron-follower chains. Length should increase with population.

---

## Phase 2: Energy Model with Types and EROEI ✅ COMPLETE

**Goal:** Replace the single `energy_endowment` scalar with distinct energy
sources that have different surplus profiles and transition dynamics.

**Status:** Implemented in `crates/walrus-engine/src/agents.rs`. Energy landscape
with 4 source types (biomass, agriculture, fossil, renewable) replaces flat
`resource_regen`. Each source has EROEI dynamics with depletion tracking,
tech-gated access via mean agent innovation, and spatial variation per grid cell.
Configurable via `EnergyParams` struct with env var overrides in the example
runner. Biomass regenerates; fossil depletes; agriculture requires fertile cells;
renewable requires high tech. Innovation grows per-tick via `innovation_growth_rate`
in `LifecycleParams`, enabling emergent tech transitions.

### 2.1 Energy Types

```
struct EnergyLandscape {
    biomass: EnergySource,      // wood, animal, plant matter
    agriculture: EnergySource,  // cultivated crops (requires sedentism)
    fossil: EnergySource,       // coal, oil, gas (requires extraction tech)
    renewable: EnergySource,    // wind, solar (requires advanced tech)
}

struct EnergySource {
    stock: f64,           // available reserve (infinite for renewables)
    flow_rate: f64,       // current extraction/harvest rate
    eroei: f64,           // energy return on energy invested
    tech_threshold: f32,  // minimum tech level to access this source
    depletion: f64,       // cumulative extraction damage
    pollution: f64,       // waste/damage per unit extracted
}
```

### 2.2 EROEI Dynamics

Each energy source has declining EROEI as the best deposits are used first:

```
eroei(t) = base_eroei * (1 - depletion)^steepness
```

- **Biomass**: base EROEI ~5:1, declines with local deforestation.
- **Agriculture**: base EROEI ~10:1, requires investment in land clearing,
  irrigation. Enables surplus storage.
- **Fossil**: base EROEI starts ~100:1 (easy oil), declines to ~5:1 as
  cheap reserves deplete.
- **Renewable**: EROEI ~10-20:1, does not deplete but requires high
  initial tech and capital investment.

### 2.3 Energy-Society Coupling

- Agents can only use energy sources they have the tech to access.
- `tech_threshold` maps to society-level innovation accumulation.
- Surplus from energy enables population growth, specialization, and
  institutional complexity.
- The *type* of energy shapes *what kind* of society emerges:
  - Biomass: supports bands and villages.
  - Agriculture: supports permanent settlements, hierarchy, storage.
  - Fossil: supports industrial-scale coordination, global networks.
  - Renewable: supports continued complexity but requires different
    institutional arrangements (distributed vs centralized).

### 2.4 What Emerges

- **Agricultural revolution**: societies near fertile land with biomass
  surplus cross the tech threshold for agriculture. Population booms.
  Hierarchy deepens because surplus needs guarding.
- **Fossil trap**: societies that discover fossil energy grow explosively
  but face declining EROEI. This creates the throughput-pressure
  superorganism dynamic described in the docs.
- **Transition challenge**: shifting from fossil to renewable requires
  institutional adaptation that the current lock-in resists.

### 2.5 Validation

- Societies with only biomass should plateau at village scale (~500).
- Agricultural societies should reach polity scale (~5000).
- Fossil societies should show explosive growth then overshoot.
- Compare EROEI trajectories to historical estimates (Hall & Klitgaard).

---

## Phase 3: Emergent Institutions ✅ COMPLETE

**Goal:** Replace formula-driven `institutional_centralization` and
`policy_lock_in` with institutions that emerge from agent interactions.

**Status:** Implemented in `crates/walrus-engine/src/agents.rs`. Institutions
are detected patterns, not coded structs. Coercion rate tracks voluntary vs
involuntary resource transfers. Property norms measured from intra-kin conflict
rates. Leadership detected when >50% of a kin group shares a patron. Public
goods investment: patrons split tax between personal wealth and group benefits.
Patron inheritance: children adopt mother's patron (institutional lock-in).
Institutional classification (Band/Tribe/Chiefdom/State) emerges from hierarchy
depth and population size. Configurable via `InstitutionParams`. 6 new tests.

### 3.1 Institution as Persistent Pattern

An institution is not a struct we create. It's a *pattern we detect*:

- **Leadership**: when >50% of a group's agents share the same patron,
  that patron is a leader. Detected, not assigned.
- **Property norms**: when agents consistently respect resource claims
  (low theft rate), property norms exist. Measured from conflict patterns.
- **Hierarchy depth**: count the longest patron-of-patron chain.
  Depth 1 = band. Depth 2 = chiefdom. Depth 3+ = state-like.
- **Specialization index**: Shannon entropy of skill_type distribution.
  Low entropy = specialized. High = generalist.
- **Coercion level**: fraction of resource transfers that are non-voluntary
  (taken by conflict or patron extraction).

### 3.2 How Hierarchy Emerges

1. Agent A is good at coordination (high cooperation, high status).
2. Neighbors delegate decisions to A (patron relationship).
3. A now controls more resources (patron extracts small tax).
4. A can invest in public goods (defense, infrastructure).
5. A's group outcompetes uncoordinated groups.
6. Natural selection preserves hierarchical groups.

No code says "if population > 150, add hierarchy." Instead:
- Delegation is an agent-level action (choose to follow a patron).
- Agents choose patrons based on prestige, resources, and group success.
- Patron extraction creates inequality.
- Inequality enables investment enables group advantage.

### 3.3 How Institutions Lock In

- Agents inherit patron relationships from parents (cultural transmission).
- Successful institutions persist because children of successful groups
  adopt the same norms.
- Lock-in occurs when the cost of *leaving* an institution exceeds the
  cost of *staying*, even when it's suboptimal. This happens naturally
  when infrastructure and surplus depend on the current arrangement.

### 3.4 Detection Metrics (replaces current formulae)

We keep the existing `EmergenceOrderParameters` as *diagnostic sensors*
applied to the emergent state:

```
fn detect_emergence(population: &Population) -> EmergenceOrderParameters {
    let hierarchy_depth = measure_patron_chain_depth(population);
    let specialization = measure_skill_entropy(population);
    let inequality = measure_gini(population);
    let coercion = measure_involuntary_transfer_rate(population);
    let coupling = measure_inter_group_trade_rate(population);

    // Map measured quantities to the existing order parameter space
    EmergenceOrderParameters {
        throughput_pressure: f(surplus_growth_rate, extraction_rate),
        coordination_centralization: f(hierarchy_depth, specialization),
        policy_lock_in: f(norm_stability, institutional_age),
        autonomy_loss: f(coercion, inequality, hierarchy_depth),
        superorganism_index: composite(above),
    }
}
```

### 3.5 Validation

- Run a population from 30 to 3000 agents over many generations.
  Hierarchy depth should increase non-linearly around Dunbar thresholds
  *without* those thresholds being coded.
- Compare detected institution types to anthropological typology:
  Band (30-50) → Tribe (150-500) → Chiefdom (1000-5000) → State (5000+).

---

## Phase 4: Inter-Society Interactions ✅ COMPLETE

> **Status (2026-03-08):** Implemented in `agents.rs` with `InterSocietyParams`,
> `TributeRelation`, `InterSocietySummary`, and `inter_society_tick()`. Mechanics:
> kin-group raids (power-based with aggression threshold), conquest triggering
> tribute relations, tribute collection/distribution per tick, resource-stress
> migration between kin groups, inter-group trade rate tracking. 5 new tests,
> 7 new CSV output columns, all env-configurable via `agent_simulation` example.

**Goal:** Societies don't just evolve in isolation on continents. They
trade, raid, conquer, form alliances, and exchange people.

### 4.1 Interaction Types

```
enum SocietyInteraction {
    Trade {
        exporter: SocietyId,
        importer: SocietyId,
        goods: ResourceBundle,
        terms: TradeTerms,
    },
    Raid {
        attacker: SocietyId,
        defender: SocietyId,
        warriors_committed: u32,
    },
    Alliance {
        members: Vec<SocietyId>,
        purpose: AlliancePurpose,  // Defense | Trade | Tribute
    },
    Migration {
        source: SocietyId,
        destination: SocietyId,
        migrants: u32,
    },
    Tribute {
        vassal: SocietyId,
        overlord: SocietyId,
        extraction_rate: f64,
    },
}
```

### 4.2 War and Conquest

- **Raid**: small-scale theft. Warriors from one group attack another.
  Outcome depends on warrior count, skill, and defender preparation.
- **Conquest**: if an attacker overwhelmingly wins, they can absorb
  the defender's population (forced migration + tribute).
- **Empire formation**: successful conquering societies accumulate
  tribute-paying vassals. This is a detected pattern, not a mode.

War creates selection pressure for hierarchy and warrior specialization.
Societies with more warriors and better coordination win more raids.
This drives militarization, which drives hierarchy, which drives the
superorganism dynamic.

### 4.3 Trade Networks

- Agents on the boundaries of societies can trade with agents from
  other societies.
- Trade creates interdependence (coupling).
- Trade routes create infrastructure that locks in relationships.
- Disrupted trade causes stress, potentially triggering conflict.

### 4.4 Validation

- Place two equal societies adjacent. One should NOT always conquer the
  other -- outcomes should depend on institutional structure and
  resource endowment.
- Trade should increase surplus for both parties (comparative advantage).
- Empires should form and collapse in cyclic patterns (Turchin-like
  secular cycles).

---

## Phase 5: Cultural Transmission and Social Structure ✅ COMPLETE

> **Status (2026-03-08):** Implemented in `agents.rs`. Replaced `norms: u64`
> bitfield with rich `Culture` struct containing kinship system, marriage rule,
> residence rule, inheritance rule (discrete enums), plus authority_norm,
> coercion_tolerance, sharing_norm, property_norm, trust_outgroup, risk_tolerance
> (continuous f32), and techniques bitfield. Three transmission mechanisms:
> vertical (parent→child with blending and mutation), horizontal (peer→peer
> random trait adoption), oblique (prestige-biased trait and technique spread).
> Cultural traits modulate behavior: sharing_norm→cooperation, trust_outgroup→trade,
> coercion_tolerance→conflict, authority_norm→delegation willingness. 9 new CSV
> columns, `CulturalParams` with 9 configurable parameters, 6 new tests.

**Goal:** Agents transmit beliefs, norms, and techniques between
generations and across societies, enabling cumulative culture and
social structure evolution.

### 5.1 Cultural Traits

Expand the `norms: u64` bitfield into a richer cultural genome:

```
struct Culture {
    // Kinship and family structure
    kinship_system: KinshipSystem,  // Patrilineal | Matrilineal | Bilateral
    marriage_rule: MarriageRule,    // Monogamy | Polygyny | Polyandry
    residence_rule: ResidenceRule,  // Patrilocal | Matrilocal | Neolocal
    inheritance_rule: InheritanceRule, // Primogeniture | Partible | Matrilineal

    // Authority and governance
    authority_norm: f32,    // 0=egalitarian, 1=highly hierarchical
    coercion_tolerance: f32, // how much extraction people accept

    // Economic
    sharing_norm: f32,      // 0=individualist, 1=full communal sharing
    property_norm: f32,     // 0=communal, 1=private property

    // Technology
    techniques: u64,        // bitfield of known technologies

    // Beliefs
    trust_outgroup: f32,    // willingness to cooperate with strangers
    risk_tolerance: f32,    // willingness to try new things
}
```

### 5.2 Transmission Mechanisms

1. **Vertical**: parent to child. High fidelity, slow.
2. **Horizontal**: peer to peer. Medium fidelity, medium speed.
3. **Oblique**: prestigious individual to many. Low fidelity, fast.
   (This is how innovations spread through prestige bias.)

Each transmission event has a mutation probability -- the cultural
analog of genetic mutation.

### 5.3 How Patriarchy Emerges (example)

Patriarchy is not hardcoded. It can emerge when:

1. Agricultural surplus creates storable wealth.
2. Wealth inheritance becomes advantageous (your children survive better
   with inherited land/cattle).
3. Paternity certainty becomes important for inheritance.
4. Patrilocal residence + patrilineal inheritance co-evolve because
   men who control land want sons nearby.
5. Female autonomy decreases as property and violence concentrate
   in male hands.

This emerges from the interaction of:
- `inheritance_rule` (cultural trait, transmitted and selected)
- `residence_rule` (cultural trait)
- `marriage_rule` (cultural trait)
- Agriculture (energy source that creates storable surplus)
- War (selects for male warrior specialization)

Whether patriarchy always emerges is an *experimental question* -- run
the simulation under different energy/geography conditions and observe.
In forager societies with low surplus, we might see egalitarian or
matrilineal outcomes persist.

### 5.4 Validation

- Forager societies should show more diverse kinship systems.
- Agricultural societies should converge toward patrilineal systems
  more often (but not always -- test with many seeds).
- Matrilineal systems should persist more in horticulture contexts
  (moderate surplus, women control food production).
- Compare cultural diversity metrics across isolation levels --
  isolated societies should diverge more.

---

## Phase 6: Integration and the Superorganism Question

**Goal:** Run the full model and measure whether superorganism
emergence is actually inevitable.

### 6.1 What "Superorganism" Means in the New Model

With real agents, the superorganism is no longer a formula. It's a
detectable macro pattern where:

1. Individual agents optimize locally (maximize their own
   resources/status/reproduction).
2. The aggregate effect is a system that maximizes throughput growth
   at the expense of long-term resilience.
3. No individual agent intends or controls this outcome.
4. The pattern is self-reinforcing (hard to exit even when individuals
   recognize the problem).

Detection: measure whether the system exhibits coordinated
throughput-maximizing behavior that persists even when individual
agents would benefit from defecting.

### 6.2 The Experiment

Run the convergence experiment from Phase 0 but with real agents:

- 8+ starting conditions (resource levels, geography, group sizes).
- 50+ seeds per condition.
- 1000+ generations per run.
- Measure: Does coordinated throughput-maximizing behavior emerge?
  How often? Under what conditions? How fast?

### 6.3 New Questions We Can Answer

- Does patriarchy always co-evolve with agriculture?
- Does hierarchy depth predict collapse fragility?
- Can egalitarian societies sustain complexity?
- Does fossil energy access always produce superorganism dynamics?
- Can renewable transitions happen without institutional collapse?

---

## Implementation Sequence

```
Phase 1: Individual Agents          ~3-4 weeks
  1.1  Agent struct + SoA layout
  1.2  Basic interactions (cooperate, trade, conflict)
  1.3  Lifecycle (birth, aging, death)
  1.4  Courtship / sexual selection
  1.5  Delegation / proto-hierarchy
  1.6  Validation suite

Phase 2: Energy Types               ~2 weeks
  2.1  EnergySource struct + EROEI
  2.2  Tech threshold gating
  2.3  Energy-society coupling
  2.4  Validation against historical EROEI

Phase 3: Emergent Institutions      ~2-3 weeks
  3.1  Detection metrics (hierarchy depth, specialization, coercion)
  3.2  Wire detection into EmergenceOrderParameters
  3.3  Remove formula-driven emergence (replace with measurement)
  3.4  Validation against anthropological typology

Phase 4: Inter-Society              ~2-3 weeks
  4.1  Trade between adjacent populations
  4.2  Raiding and warfare
  4.3  Conquest and tribute
  4.4  Alliance formation
  4.5  Validation (secular cycles, empire dynamics)

Phase 5: Cultural Transmission      ~2-3 weeks
  5.1  Culture struct + transmission mechanisms
  5.2  Kinship / marriage / residence / inheritance rules
  5.3  Prestige-biased transmission
  5.4  Validation (kinship system distribution)

Phase 6: Integration                ~2 weeks
  6.1  Full convergence experiment with real agents
  6.2  Superorganism detection from emergent metrics
  6.3  Result analysis and visualization
```

## Dependencies

```
Phase 1 ──> Phase 2 (agents need energy to interact with)
Phase 1 ──> Phase 3 (institutions emerge from agent interactions)
Phase 2 ──> Phase 3 (energy type shapes institution type)
Phase 1 ──> Phase 4 (war/trade requires individual agents)
Phase 3 ──> Phase 5 (cultural traits include institutional norms)
All    ──> Phase 6 (integration requires all components)
```

Phase 2 and Phase 4 can be developed in parallel after Phase 1.
Phase 3 and Phase 5 can overlap.

## Risks

1. **Performance**: 1M+ agents with O(N) interactions per tick may be
   too slow. Mitigation: spatial partitioning, interaction budgets,
   SoA layout.
2. **Parameter explosion**: many new agent traits = many new knobs.
   Mitigation: sensitivity analysis after each phase. Remove traits
   that don't affect outcomes.
3. **Emergence may not happen**: agents might produce boring dynamics
   or chaotic noise. Mitigation: validate each phase independently.
   If Phase 1 doesn't produce Dunbar-like grouping, fix before
   proceeding.
4. **Scope creep**: each phase could expand indefinitely.
   Mitigation: strict validation gates. Phase N+1 doesn't start
   until Phase N passes its validation suite.

## Relationship to Existing Code

- `lib.rs` functions (`emergence_order_parameters`, `group_behavior_profile`,
  etc.) become *diagnostic instruments* applied to the new population state.
  They are not deleted -- they become the measurement layer.
- `evolution.rs` `simulate_evolution` becomes the reference baseline.
  The new simulation runs alongside it for comparison.
- `calibration.rs` and `ensemble.rs` remain unchanged -- they operate on
  time series that both old and new simulations can produce.
- The convergence experiment framework stays -- it just runs the new
  simulation instead of (or alongside) the old one.
