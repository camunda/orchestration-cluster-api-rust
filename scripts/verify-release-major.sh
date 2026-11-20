#!/usr/bin/env bash
# Fails unless the version about to be released belongs to the branch's line, as
# semantic-release's verifyRelease step, so a wrong major stops before any tag is pushed.
#
# The SDK major tracks the Camunda server minor (server 8.10 -> SDK 10.x):
#   - stable/N releases N.x.y only (never a prerelease, never another major).
#   - main releases prereleases of the next line only (e.g. 11.0.0-alpha.N).
#
# Usage: scripts/verify-release-major.sh <version> [branch]
#   branch defaults to $GITHUB_REF_NAME, then the current git branch.
set -euo pipefail

version="${1:?usage: $0 <version> [branch]}"
version="${version#v}"
branch="${2:-${GITHUB_REF_NAME:-$(git rev-parse --abbrev-ref HEAD)}}"

major="${version%%.*}"
if ! [[ "$major" =~ ^[0-9]+$ ]]; then
  echo "::error::'$1' is not a SemVer version." >&2
  exit 1
fi
prerelease=""
[[ "$version" == *-* ]] && prerelease="${version#*-}"

echo "version=$version branch=$branch major=$major prerelease=${prerelease:-<none>}"
if [[ "$branch" =~ ^stable/([0-9]+)$ ]]; then
  line="${BASH_REMATCH[1]}"
  if [ "$major" != "$line" ]; then
    echo "::error::$branch may only release ${line}.x, not $version. A breaking change on a stable branch must not bump the major." >&2
    exit 1
  fi
  if [ -n "$prerelease" ]; then
    echo "::error::$branch must not release the prerelease $version." >&2
    exit 1
  fi
elif [ "$branch" = "main" ]; then
  if [ -z "$prerelease" ]; then
    echo "::error::main releases prereleases of the next line only, not $version." >&2
    exit 1
  fi
else
  echo "::error::Releases are cut only from main or stable/N, not '$branch'." >&2
  exit 1
fi
