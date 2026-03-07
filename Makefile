.PHONY: fmt lint test coverage coverage-engine coverage-workspace check feedback-loop system-feedback sim-sweep viz-report viz-app tui-life calibration-loop evolution-run evolution-sweep convergence-experiment agent-sim

LLVM_COV_BIN := $(shell sh -c 'command -v llvm-cov 2>/dev/null || xcrun --find llvm-cov 2>/dev/null')
LLVM_PROFDATA_BIN := $(shell sh -c 'command -v llvm-profdata 2>/dev/null || xcrun --find llvm-profdata 2>/dev/null')

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace --all-targets

coverage-engine:
	LLVM_COV="$(LLVM_COV_BIN)" LLVM_PROFDATA="$(LLVM_PROFDATA_BIN)" cargo llvm-cov --package walrus-engine --all-targets --fail-under-lines 90 --summary-only

coverage-workspace:
	LLVM_COV="$(LLVM_COV_BIN)" LLVM_PROFDATA="$(LLVM_PROFDATA_BIN)" cargo llvm-cov --workspace --all-targets --fail-under-lines 80 --summary-only

coverage: coverage-engine coverage-workspace

check:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace --all-targets

feedback-loop:
	$(MAKE) check
	$(MAKE) coverage-engine

system-feedback:
	cargo test -p walrus-engine --test system_feedback
	cargo run -q -p walrus-engine --example emergence_run

sim-sweep:
	cargo run -q -p walrus-engine --example sweep_scenarios

viz-report:
	cargo run -q -p walrus-engine --example visualization_report

viz-app:
	cargo run -q -p walrus-engine --example visualization_app

tui-life:
	cargo run -q -p walrus-engine --example agent_life_tui

calibration-loop:
	$(MAKE) check
	cargo run -q -p walrus-engine --example visualization_report
	cargo run -q -p walrus-engine --example visualization_app

evolution-run:
	cargo run -q -p walrus-engine --example evolutionary_map_run

evolution-sweep:
	cargo run -q -p walrus-engine --example evolution_isolation_sweep

convergence-experiment:
	cargo run -q --release -p walrus-engine --example convergence_experiment

agent-sim:
	cargo run -q --release -p walrus-engine --example agent_simulation
