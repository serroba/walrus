# Plan 06: Language and Stack Selection

## Decision Criteria

1. Runtime performance and energy efficiency.
2. Memory safety and long-term maintainability.
3. Parallel/distributed ecosystem maturity.
4. Scientific workflow interoperability.
5. Contributor accessibility.

## Candidate Architecture

1. Engine core: Rust.
2. Experiment orchestration and analysis: Python.
3. Data format: Arrow/Parquet.
4. Optional acceleration: GPU path later via specialized crates/libraries.

## Rationale

- Rust provides high performance, memory safety, and deterministic systems control.
- Python keeps research iteration fast and accessible.
- Arrow/Parquet enables large-scale IO and cross-tool compatibility.

## Decision Milestone

Finalize stack after MVP profiling spike and contributor trial task.

## Exit Criteria

- Prototype benchmark completed.
- Developer experience documented.
- Initial contributors can add one model module end-to-end.
