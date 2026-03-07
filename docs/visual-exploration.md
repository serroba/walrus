# Visual Exploration

These visuals are generated from simulation timeline data in `outputs/latest/timeline_*.csv`.

## Baseline

![Baseline trajectory](./assets/snapshot_baseline_default.svg)

## Ecological Stress (Fragile)

![Ecological stress trajectory](./assets/snapshot_eco-stress_fragile.svg)

## Fragmented Low Coupling

![Fragmented low coupling trajectory](./assets/snapshot_fragmented-low-coupling_default.svg)

## How to Regenerate

1. `make sim-sweep`
2. `make viz-report`
3. `node scripts/generate_snapshots.mjs`
4. `make viz-app`
