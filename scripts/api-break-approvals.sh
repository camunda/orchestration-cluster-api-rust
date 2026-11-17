#!/usr/bin/env bash
# Usage: api-break-approvals.sh <baseline-tag> <release-branch>
#
# Prints `#N` for each PR labelled api-break-approved that was merged into <release-branch>
# after <baseline-tag>, one per line. A label on a PR merged elsewhere (say `main`, whose
# commits a backport shares) or before the baseline approves nothing in this release.
# Exits non-zero if a lookup fails, so the caller can tell "none" from "unknown".
set -euo pipefail

tag="$1"
branch="$2"
shas=$(git log --format=%H "${tag}..HEAD")

for sha in $shas; do
  gh api "repos/${GITHUB_REPOSITORY}/commits/${sha}/pulls" \
    --jq '.[] | select(.merged_at != null) | select(any(.labels[]; .name == "api-break-approved")) | "\(.number) \(.base.ref) \(.merge_commit_sha)"' \
    | while read -r number base merge_sha; do
        if [ "$base" = "$branch" ] && grep -qx "$merge_sha" <<<"$shas"; then
          echo "#${number}"
        fi
      done
done | sort -u
