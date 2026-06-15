#!/usr/bin/env bash
#
# Export a self-contained copy of the SDK (ergonomic runtime crate + generated
# low-level client) into a destination directory, suitable for committing into a
# downstream repo that consumes the SDK via a Cargo **path dependency**
# (vendoring). The vendored copy is a standalone Cargo workspace that builds
# offline with no reference back to this repo.
#
# Usage:
#   scripts/vendor.sh <dest-dir>
#   make vendor DEST=<dest-dir>
#
set -euo pipefail

DEST="${1:?usage: vendor.sh <dest-dir>}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Resolve DEST to an absolute path (it may not exist yet).
mkdir -p "$DEST"
DEST="$(cd "$DEST" && pwd)"

echo "==> Vendoring SDK from $ROOT"
echo "    into $DEST"

# Start clean so removed files upstream don't linger in the vendored copy.
rm -rf "$DEST"
mkdir -p "$DEST/client"

# Runtime crate (workspace + package root).
cp "$ROOT/Cargo.toml" "$DEST/Cargo.toml"
cp -R "$ROOT/src" "$DEST/src"
[ -f "$ROOT/README.md" ] && cp "$ROOT/README.md" "$DEST/README.md"
[ -f "$ROOT/LICENSE" ] && cp "$ROOT/LICENSE" "$DEST/LICENSE"

# Generated low-level client crate (workspace member).
cp "$ROOT/client/Cargo.toml" "$DEST/client/Cargo.toml"
cp -R "$ROOT/client/src" "$DEST/client/src"

# Record provenance so consumers can tell which SDK commit was vendored.
{
  echo "# Vendored copy of camunda-orchestration-sdk — DO NOT EDIT BY HAND."
  echo "# Refresh with the SDK repo's: make vendor DEST=<this-dir>"
  printf 'commit: '
  git -C "$ROOT" rev-parse HEAD 2>/dev/null || echo "unknown"
  printf 'vendored_at: '
  date -u +"%Y-%m-%dT%H:%M:%SZ"
} > "$DEST/VENDOR.md"

echo "==> Done. Vendored:"
echo "      Cargo.toml, src/, client/Cargo.toml, client/src/, VENDOR.md"
