.PHONY: help bundle generate build test lint fmt fmt-check clean

SPEC_REF ?= main

help:
	@echo "Camunda Orchestration Cluster API — Rust SDK"
	@echo ""
	@echo "Targets:"
	@echo "  make bundle      Re-bundle the upstream OpenAPI spec (ref: $(SPEC_REF)) via camunda-schema-bundler"
	@echo "  make generate    Regenerate the client crate from the bundled spec + run domain-type post-processing"
	@echo "  make build       Build the whole workspace"
	@echo "  make test        Run all tests (unit + doctests)"
	@echo "  make lint        Run clippy"
	@echo "  make fmt         Format all crates"
	@echo "  make fmt-check   Check formatting"
	@echo "  make clean       Remove build artifacts"

# Re-bundle the upstream spec AND regenerate.
bundle:
	./scripts/generate.sh --bundle

# Regenerate from the already-bundled spec (no network fetch).
generate:
	./scripts/generate.sh

build:
	cargo build --workspace

test:
	cargo test --workspace

lint:
	cargo clippy --workspace

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clean:
	cargo clean
