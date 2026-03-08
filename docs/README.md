# Walrus Simulator Docs

This folder defines the foundation and execution plans for an open-source, high-performance simulation project to test system-level assumptions (energy, materials, ecology, institutions, behavior) and study emergence of a global economic **superorganism**.

## Structure

- `foundations/`: explicit assumptions, mathematical model, and simulation design.
- `plans/`: phased implementation plans and delivery milestones.

## Guiding Principles

1. Assumptions-first: every model claim must be explicit, versioned, and replaceable.
2. Multi-scale: individuals, institutions, sectors, and global aggregates.
3. Reproducibility: deterministic seeds, scenario manifests, and archived outputs.
4. Performance portability: laptop-first, cluster-ready architecture.
5. Open science: transparent equations, uncertainty ranges, and validation protocol.

## Current Modeling Style

The primary simulation layer is the **agent-based model** (`agents.rs`), where individual agents interact locally and macro patterns emerge bottom-up. The 6-phase implementation plan ([`plans/08_emergent-society-evolution.md`](./plans/08_emergent-society-evolution.md)) is complete:

1. Individual agents with traits (SoA layout, rayon parallelism)
2. Energy model with EROEI dynamics and tech-gated transitions
3. Emergent institutions (detected, not hardcoded)
4. Inter-society interactions (raids, conquest, tribute, migration)
5. Cultural transmission (kinship, marriage, norms, techniques)
6. Superorganism detection and convergence experiment

Legacy layers (`lib.rs` stock-flow model, `evolution.rs` actor model) remain as diagnostic instruments and alternative modeling approaches.

## Key Documents

| Document | Purpose |
|----------|---------|
| [`plans/08_emergent-society-evolution.md`](./plans/08_emergent-society-evolution.md) | 6-phase agent simulation plan (all complete) |
| [`foundations/01_scope-and-goals.md`](./foundations/01_scope-and-goals.md) | Project scope and goals |
| [`foundations/06_emergence-rules-and-feedbacks.md`](./foundations/06_emergence-rules-and-feedbacks.md) | Emergence-first design principles |
| [`foundations/09_agent-actor-simulation.md`](./foundations/09_agent-actor-simulation.md) | Agent/actor simulation usage guidance |
| [`foundations/10_mental-frameworks-and-assumptions.md`](./foundations/10_mental-frameworks-and-assumptions.md) | Modeling assumptions and mental frameworks |
| [`foundations/12_moloch-ai-coordination-framework.md`](./foundations/12_moloch-ai-coordination-framework.md) | Coordination-failure and AI-risk framing |
| [`validation/01_assumption-validation.md`](./validation/01_assumption-validation.md) | Assumption coverage and readiness review |
