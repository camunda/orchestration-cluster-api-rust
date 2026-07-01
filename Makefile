.PHONY: help bundle generate build test lint fmt fmt-check clean vendor examples sync-readme sync-readme-check check publish-dry-run

SPEC_REF ?= main

help:
	@echo "Camunda Orchestration Cluster API — Rust SDK"
	@echo ""
	@echo "Targets:"
	@echo "  make bundle      Re-bundle the upstream OpenAPI spec (ref: $(SPEC_REF)) via camunda-schema-bundler"
	@echo "  make generate    Regenerate the client crate from the bundled spec + run domain-type post-processing"
	@echo "  make vendor DEST=<dir>  Export a self-contained SDK copy for vendoring into a downstream repo"
	@echo "  make build       Build the whole workspace"
	@echo "  make test        Run all tests (unit + doctests)"
	@echo "  make examples    Type-check the example programs (incl. README snippet sources)"
	@echo "  make lint        Run clippy"
	@echo "  make fmt         Format all crates"
	@echo "  make fmt-check   Check formatting"
	@echo "  make sync-readme        Inject example snippets into README.md"
	@echo "  make sync-readme-check  Verify README snippets are in sync (CI mode)"
	@echo "  make check       Run the full CI gate (build, test, examples, lint, fmt, README sync)"
	@echo "  make publish-dry-run  Package + verify both crates for crates.io without uploading"
	@echo "  make clean       Remove build artifacts"

# Re-bundle the upstream spec AND regenerate.
bundle:
	./scripts/generate.sh --bundle

# Regenerate from the already-bundled spec (no network fetch).
generate:
	./scripts/generate.sh

# Export a self-contained copy of the SDK for vendoring into a downstream repo.
# Usage: make vendor DEST=../some-repo/path/to/camunda-orchestration-sdk
vendor:
	./scripts/vendor.sh "$(DEST)"

build:
	cargo build --workspace

test:
	cargo test --workspace

# Type-check the example programs. The README snippets are injected from these files
# (examples/readme.rs), so a green build guarantees the README cannot drift from the API.
examples:
	cargo build --examples

lint:
	cargo clippy --workspace --all-targets

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

# Inject region-tagged snippets from examples/*.rs into README.md.
sync-readme:
	python3 scripts/sync-readme-snippets.py

# CI mode: fail if README.md is out of sync with the example snippets, or if any
# rust code block in the README is not backed by a compilable example.
sync-readme-check:
	python3 scripts/sync-readme-snippets.py --check

# Full local CI gate.
check: build test examples lint fmt-check sync-readme-check

# Package + verify both workspace crates for crates.io without uploading. Mirrors
# what the release workflow runs; publishes client first then the SDK, resolving
# the path dependency against a temp registry.
publish-dry-run:
	cargo publish --workspace --locked --dry-run

clean:
	cargo clean
