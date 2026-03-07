# Plan 02: MVP Simulator

## Goal

Ship a minimal agent-based simulator demonstrating macro emergence from micro rules.

## Scope

1. 3-4 agent classes.
2. 2 resource stocks (one renewable, one non-renewable).
3. 1 governance module with adaptive policy.
4. 1 stress channel (resource -> price -> legitimacy).
5. Group-size and subsistence-regime transitions (hunter-gatherer -> sedentary -> agriculture).

## Deliverables

1. Tick-based engine with event queue.
2. Baseline scenario and three counterfactuals.
3. Core dashboards (CLI plots + exported notebook template).
4. Calibration stubs with documented priors.

## Acceptance Criteria

- Simulates at least 1M agents on developer hardware in reasonable time budget.
- Produces interpretable regime transitions under scenario changes.
- Includes regression tests on key metrics.
