#!/usr/bin/env bash
#
# generate.sh — Regenerate the low-level client crate from the bundled OpenAPI spec.
#
# Pipeline:
#   1. (optional) bundle:spec  — fetch + bundle upstream spec via camunda-schema-bundler
#   2. openapi-generator         — generate the `client/` crate (reqwest + serde, async)
#   3. postprocess              — run the numbered hooks: Domain Type System, semantic
#                                  field types, known openapi-generator output fixes, and
#                                  the flat full-surface ergonomic facade
#
# Usage:
#   ./scripts/generate.sh            # generate from the existing bundled spec
#   ./scripts/generate.sh --bundle   # re-bundle the upstream spec first (ref: $SPEC_REF, default main)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SPEC_REF_DEFAULT="main"
SPEC_REF="${SPEC_REF:-$SPEC_REF_DEFAULT}"
BUNDLED_SPEC="external-spec/bundled/rest-api.bundle.json"
BUNDLED_META="external-spec/bundled/spec-metadata.json"

if [[ "${1:-}" == "--bundle" ]]; then
  echo "==> Bundling upstream spec (ref: ${SPEC_REF}) via camunda-schema-bundler..."
  npx --yes camunda-schema-bundler@^2.4.3 --ref "$SPEC_REF" \
    --output-spec "$BUNDLED_SPEC" \
    --output-metadata "$BUNDLED_META"
fi

echo "==> Generating Rust client crate with openapi-generator..."
npx --yes @openapitools/openapi-generator-cli generate \
  -c openapi-generator-config.yaml

echo "==> Post-processing: Domain Type System + semantic fields + generator fixups..."
python3 scripts/postprocess.py \
  --client-dir client \
  --spec "$BUNDLED_SPEC" \
  --metadata "$BUNDLED_META"

echo "==> Formatting generated crate..."
( cd client && cargo fmt 2>/dev/null || true )

echo "==> Done. Verifying it builds..."
( cd client && cargo build )
echo "==> Client crate generated and builds cleanly."
