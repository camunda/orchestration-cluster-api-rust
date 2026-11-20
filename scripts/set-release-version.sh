#!/usr/bin/env bash
# Sets the version of both workspace crates, as semantic-release's prepare step.
#
# Both crates share one version, written to:
#   - Cargo.toml                    [package] version and the client path dependency
#   - client/Cargo.toml             [package] version
#   - openapi-generator-config.yaml packageVersion, so regeneration keeps the bump
#   - client/README.md              the generator renders packageVersion into it, so
#                                   leaving it stale fails the generation-drift check
# and Cargo.lock is refreshed to match.
#
# Usage: scripts/set-release-version.sh <version>   (e.g. 10.0.1, 11.0.0-alpha.2)
set -euo pipefail

version="${1:?usage: $0 <version>}"
version="${version#v}"
if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  echo "::error::'$1' is not a SemVer version." >&2
  exit 1
fi

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

python3 - "$version" <<'PY'
import re
import sys
from pathlib import Path

version = sys.argv[1]


def sub_once(path, pattern, repl):
    p = Path(path)
    text = p.read_text()
    new, n = re.subn(pattern, repl, text, count=1, flags=re.MULTILINE)
    if n != 1:
        sys.exit(f"::error::{path}: expected one match for {pattern!r}")
    p.write_text(new)


# The first `version =` line is the [package] version in both manifests.
sub_once("Cargo.toml", r'^version = "[^"]*"', f'version = "{version}"')
sub_once(
    "Cargo.toml",
    r'^(camunda-orchestration-api-client = \{[^}\n]*version = ")[^"]*(")',
    rf"\g<1>{version}\g<2>",
)
sub_once("client/Cargo.toml", r'^version = "[^"]*"', f'version = "{version}"')
sub_once(
    "openapi-generator-config.yaml",
    r"^(\s*packageVersion:\s*)\S+",
    rf"\g<1>{version}",
)
sub_once(
    "client/README.md",
    r"^(- Package version: )\S+",
    rf"\g<1>{version}",
)
PY

cargo update --workspace --offline 2>/dev/null || cargo update --workspace

# Fail if any manifest disagrees with the requested version.
actual="$(cargo metadata --no-deps --format-version 1 \
  | python3 -c "import json,sys; print(' '.join(sorted({p['version'] for p in json.load(sys.stdin)['packages']})))")"
if [ "$actual" != "$version" ]; then
  echo "::error::Workspace versions are '$actual' after setting $version." >&2
  exit 1
fi
echo "Set both crates to $version."
