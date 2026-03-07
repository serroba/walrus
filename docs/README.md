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

## Immediate Next Step

Start with [`foundations/01_scope-and-goals.md`](./foundations/01_scope-and-goals.md), then implement `plans/01_project-bootstrap.md` and `plans/02_mvp-simulator.md`.
For emergence-first work, also use [`foundations/06_emergence-rules-and-feedbacks.md`](./foundations/06_emergence-rules-and-feedbacks.md).
For macro stock-flow constraints, use [`foundations/07_pyworld3-integration-notes.md`](./foundations/07_pyworld3-integration-notes.md).
For non-technical communication, use [`foundations/08_visualization-guidelines.md`](./foundations/08_visualization-guidelines.md).
For concrete agent/actor simulation usage, use [`foundations/09_agent-actor-simulation.md`](./foundations/09_agent-actor-simulation.md).
For generated example visuals, use [`visual-exploration.md`](./visual-exploration.md).
For assumption coverage and readiness review, use [`validation/01_assumption-validation.md`](./validation/01_assumption-validation.md).
