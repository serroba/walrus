# Plan 02: MVP Simulator

## Goal

Ship a minimal agent-based simulator demonstrating macro emergence from micro rules.

## Scope

1. 3-4 agent classes.
2. 2 resource stocks (one renewable, one non-renewable).
3. 1 governance module with adaptive policy.
4. 1 stress channel (resource -> price -> legitimacy).
5. Group-size and subsistence-regime transitions (hunter-gatherer -> sedentary -> agriculture).
6. Multi-society world: many local societies with local emergence metrics before global aggregation.

## Deliverables

1. Tick-based engine with event queue.
2. Baseline scenario and three counterfactuals.
3. Core dashboards (CLI plots + exported notebook template).
4. Calibration stubs with documented priors.
5. Emergence telemetry: throughput pressure, centralization, policy lock-in, autonomy loss, superorganism index.
6. Dynamic transition runner: local societies evolve over time with mode switching and ecological balancing feedback.

## Acceptance Criteria

- Simulates at least 1M agents on developer hardware in reasonable time budget.
- Produces interpretable regime transitions under scenario changes.
- Includes regression tests on key metrics.
