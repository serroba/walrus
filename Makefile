.PHONY: fmt lint test coverage check

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace --all-targets

coverage:
	cargo llvm-cov --workspace --all-targets --fail-under-lines 85 --summary-only

check:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace --all-targets
