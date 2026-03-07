# 04 Agent Architecture

## Agent Types (Initial)

1. Household agents: labor, consumption, adaptation, political preference.
2. Producer agents: transform energy/materials to goods/services.
3. Financial agents: capital allocation and credit conditions.
4. Governance agents: taxation, transfer, regulation, coercion.

## Agent State Vector

- Endowments: wealth, skills, assets.
- Needs: food, shelter, mobility, security.
- Preferences: status vs security vs cooperation.
- Beliefs: trust in institutions, risk perception, ideology.
- Constraints: geography, access, policy, network position.
- Group context: current group size, social density, and subsistence mode.

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
- ecological pressure.

## Time Step Pipeline

1. Observe local + global signals.
2. Decide action.
3. Execute transactions and resource use.
4. Update stocks/flows and ecological state.
5. Update institutions and enforcement.
6. Record telemetry.
