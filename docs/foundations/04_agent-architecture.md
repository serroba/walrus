# 04 Agent Architecture

## Agent Types (Initial)

1. Household agents: labor, consumption, adaptation, political preference.
2. Producer agents: transform energy/materials to goods/services.
3. Financial agents: capital allocation and credit conditions.
4. Governance agents: taxation, transfer, regulation, coercion.

## Agent Roles

Each micro-agent carries a functional role that modifies interaction behavior:

1. **Producer** (~70% of population): baseline resource gatherers and farmers.
2. **Coordinator** (~15%): governance/coordination specialists — boost cooperation (+10%), reduce conflict (-8%), lower aggression.
3. **Trader** (~15%): exchange specialists — boost trade (+12%), higher trust and resources.

Roles are inherited at birth with a 10% mutation rate, and replacement agents draw from the same distribution.

## Agent State Vector

- Endowments: wealth, skills, assets.
- Needs: food, shelter, mobility, security.
- Preferences: status vs security vs cooperation.
- Beliefs: trust in institutions, risk perception, ideology.
- Constraints: geography, access, policy, network position.
- Group context: current group size, social density, and subsistence mode.
- **Affinity vector** (`[f64; 3]`): continuous cultural identity used by the oxytocin model (see below).

## Oxytocin Model (Affinity-Based In-Group/Out-Group Dynamics)

Each agent carries a 3-dimensional affinity vector in `[0, 1]^3`. The Euclidean distance between two agents' vectors determines whether they perceive each other as in-group or out-group, modelling oxytocin's documented dual role:

**In-group bonding** (small affinity distance < 0.45):
- +15% cooperation bias, +8% trade preference.
- Cooperation events cause affinity convergence (drift rate 0.04).
- Trade events cause mild convergence (drift rate 0.015).

**Out-group othering** (large affinity distance > 0.45):
- +14% conflict bias, -6% trade aversion, +10% migration pressure.
- Conflict events cause affinity divergence (drift rate -0.03).

**Emergent tribal dynamics:**
- Cooperating agents become culturally similar, reinforcing further cooperation (positive feedback).
- Conflict creates deepening cultural divides, producing polarization (positive feedback).
- Birth inheritance with ±3% mutation per dimension provides cultural drift.
- Replacement agents arrive with random affinity (immigrant/outsider effect).
- Initial populations are seeded with ~3-4 clusters via golden-ratio spacing.

## Behavioral Engines (Pluggable)

- Rule-based heuristics (fast, interpretable).
- Utility optimization (medium realism).
- Learning agents (RL/evolutionary, high complexity).

Phase 1 uses rule-based + simple bounded utility.

## Interaction Topologies

- Local social graph.
- Market graph (buyers/sellers/credit).
- Governance graph (jurisdictions).

## Group-Level State

Each group carries:

- population size,
- subsistence mode (`HunterGatherer`, `Sedentary`, `Agriculture`),
- mobility level,
- institutional centralization,
- coercion capacity,
- ecological pressure,
- **governance state** (policy type, tax/redistribution rates, stress channel).

## Governance Module

Each local society has an adaptive governance module (`GovernanceState`) with three policy types:

1. **Laissez** — minimal taxation (5%), high redistribution share (50%). Active when legitimacy is high and surplus adequate.
2. **Redistributive** — moderate taxation (15%), high redistribution (70%). Triggered when legitimacy drops below 0.65 or surplus is thin.
3. **Extractive** — heavy taxation (25%), low redistribution (20%). Emerges when legitimacy falls below 0.35 (elite capture under crisis).

Policy selection is adaptive: the `adapt_governance` function updates policy each tick based on the stress channel state.

### Stress Channel (Resource → Price → Legitimacy)

The governance module includes an explicit causal stress channel:

1. **Resource scarcity** (ecological pressure + surplus deficit) → **price pressure** (EMA-smoothed, α=0.15).
2. Sustained **price pressure** → **legitimacy erosion** (rate: -0.08 × price_pressure per tick).
3. Low **legitimacy** → policy shift (Laissez → Redistributive → Extractive).
4. **Extractive** policy → increased ecological pressure (+0.02/tick), creating a vicious cycle.
5. When pressure subsides, legitimacy slowly recovers (+0.04 × (1 - price_pressure) per tick).

This produces boom-bust governance cycles: prosperity → laissez-faire → overshoot → scarcity → extractive capture → degradation → eventual recovery.

## Time Step Pipeline

1. Observe local + global signals.
2. Decide action.
3. Execute transactions and resource use.
4. Update stocks/flows and ecological state.
5. Update institutions and enforcement.
6. Record telemetry.
