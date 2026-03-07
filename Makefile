.PHONY: fmt lint test coverage coverage-engine coverage-workspace check

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace --all-targets

coverage-engine:
	cargo llvm-cov --package walrus-engine --all-targets --fail-under-lines 90 --summary-only

coverage-workspace:
	cargo llvm-cov --workspace --all-targets --fail-under-lines 80 --summary-only

coverage: coverage-engine coverage-workspace

check:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace --all-targets
