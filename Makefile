# Makefile for gun.rs development tasks

.PHONY: help test build fmt clippy clean all check bench

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

all: fmt clippy test ## Run all checks (format, clippy, tests)

build: ## Build the project
	cargo build --all-features

test: ## Run all tests
	cargo test --all-features

test-unit: ## Run unit tests only
	cargo test --lib

test-integration: ## Run integration tests
	cargo test --test integration_tests -- --test-threads=1

test-stress: ## Run stress tests (may take longer)
	cargo test --test stress_tests -- --ignored --test-threads=1

test-locks: ## Run lock contention tests
	cargo test --test lock_tests -- --test-threads=1

fmt: ## Format code
	cargo fmt --all

fmt-check: ## Check formatting without modifying files
	cargo fmt --all -- --check

clippy: ## Run clippy lints
	cargo clippy --all-targets --all-features -- -D warnings

clippy-fix: ## Try to auto-fix clippy issues
	cargo clippy --fix --all-targets --all-features -- -D warnings

check: fmt-check clippy ## Run format check and clippy

clean: ## Clean build artifacts
	cargo clean

clean-all: clean ## Clean build artifacts and test data
	cargo clean
	rm -rf gun_data/*

bench: ## Run benchmarks (if available)
	cargo bench

doc: ## Build documentation
	cargo doc --all-features --open

audit: ## Run security audit
	cargo audit

machete: ## Check for unused dependencies
	cargo install cargo-machete --locked || true
	cargo machete

examples: ## Build and run examples
	cargo build --examples
	@echo "Examples built. Run with: cargo run --example <name>"

server: ## Build the relay server
	cargo build --bin gun-server --release

run-server: ## Run the relay server
	cargo run --bin gun-server

ci: fmt-check clippy build test ## Run CI checks locally

