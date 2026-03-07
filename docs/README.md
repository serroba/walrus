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

1. Assumptions-first and replaceable:
- thresholds, topologies, feedbacks, and calibration objectives are explicit and configurable.
2. Actor + system coupling:
- micro actor dynamics produce macro emergence and collapse signatures.
3. Geography and isolation experiments:
- abstract continent layouts are used to test convergence vs divergence.
4. Social scaling realism:
- Dunbar effects are represented as behavioral constraints, not only fixed boundaries.

## Immediate Next Step

Start with [`foundations/01_scope-and-goals.md`](./foundations/01_scope-and-goals.md), then implement `plans/01_project-bootstrap.md` and `plans/02_mvp-simulator.md`.
For emergence-first work, also use [`foundations/06_emergence-rules-and-feedbacks.md`](./foundations/06_emergence-rules-and-feedbacks.md).
For macro stock-flow constraints, use [`foundations/07_pyworld3-integration-notes.md`](./foundations/07_pyworld3-integration-notes.md).
For non-technical communication, use [`foundations/08_visualization-guidelines.md`](./foundations/08_visualization-guidelines.md).
For concrete agent/actor simulation usage, use [`foundations/09_agent-actor-simulation.md`](./foundations/09_agent-actor-simulation.md).
For explicit modeling assumptions and mental frameworks, use [`foundations/10_mental-frameworks-and-assumptions.md`](./foundations/10_mental-frameworks-and-assumptions.md).
For macro-reference integration notes (World3/HANDY), use [`foundations/11_nasa-world3-handy-integration.md`](./foundations/11_nasa-world3-handy-integration.md).
For coordination-failure and AI-risk framing, use [`foundations/12_moloch-ai-coordination-framework.md`](./foundations/12_moloch-ai-coordination-framework.md).
For generated example visuals, use [`visual-exploration.md`](./visual-exploration.md).
For assumption coverage and readiness review, use [`validation/01_assumption-validation.md`](./validation/01_assumption-validation.md).
