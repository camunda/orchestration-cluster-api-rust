.PHONY: help bundle generate build test lint fmt fmt-check clean vendor examples sync-readme sync-readme-check check publish-dry-run docs-json docs-md print-docs-toolchain

SPEC_REF ?= main

# Toolchain used ONLY for rustdoc JSON output, which is nightly-gated. This is
# deliberately separate from rust-toolchain.toml (which pins the stable channel
# used for build/test/lint). rustdoc JSON is an unstable format: its
# `format_version` must stay in the SUPPORTED_FORMAT_VERSIONS set declared in
# scripts/generate-docusaurus-md.py, so pin an exact nightly here.
DOCS_TOOLCHAIN ?= nightly-2026-08-03

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
	@echo "  make docs-json   Emit rustdoc JSON for both crates (nightly: $(DOCS_TOOLCHAIN))"
	@echo "  make docs-md     Generate the Docusaurus markdown under docs-md/"
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

# Emit the pinned rustdoc nightly so CI can install it without duplicating the value.
print-docs-toolchain:
	@echo $(DOCS_TOOLCHAIN)

# Emit rustdoc JSON for both workspace crates. Requires the pinned nightly:
#   rustup toolchain install $(DOCS_TOOLCHAIN)
docs-json:
	cargo +$(DOCS_TOOLCHAIN) rustdoc -p camunda-orchestration-sdk --lib -- \
		-Z unstable-options --output-format json
	cargo +$(DOCS_TOOLCHAIN) rustdoc -p camunda-orchestration-api-client --lib -- \
		-Z unstable-options --output-format json

# Generate the Docusaurus markdown consumed by camunda-docs. The output is gitignored:
# the sync-rust-sdk-docs workflow in camunda/camunda-docs regenerates it and opens a PR
# against the docs site.
docs-md: docs-json
	python3 scripts/generate-docusaurus-md.py --validate-links

# Full local CI gate.
check: build test examples lint fmt-check sync-readme-check

# Package + verify both workspace crates for crates.io without uploading. Mirrors
# what the release workflow runs; publishes client first then the SDK, resolving
# the path dependency against a temp registry.
publish-dry-run:
	cargo publish --workspace --locked --dry-run

clean:
	cargo clean
