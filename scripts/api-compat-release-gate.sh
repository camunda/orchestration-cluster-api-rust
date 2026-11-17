#!/usr/bin/env bash
# Usage: api-compat-release-gate.sh <baseline-version> <major>
#
# Release gate: compares the code about to be published with <baseline-version> and passes
# if it is compatible or every break is approved. Exit 0 pass, 1 blocked, 2 setup error.
#
# An api-break-approved label approves the API as of that PR's merge. Breaks added after the
# newest approved merge (a later direct push, say) need their own approval.
set -uo pipefail

here=$(dirname "$0")
baseline="$1"
major="$2"
tag="v${baseline}"

rc=0
python3 "$here/api_compat.py" --baseline-version "$baseline" || rc=$?
[ "$rc" -eq 1 ] || exit "$rc"

if [ "${DISPATCH_APPROVED:-false}" = "true" ]; then
  echo "::warning::Breaking API changes approved by the api_break_approved input of this run."
  exit 0
fi
# A manual publish creates no tag, so only the approval search depends on it.
if ! git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
  echo "::error::Breaking API changes, and baseline ${baseline} has no ${tag} tag to list the PRs merged since. Re-run this workflow manually with api_break_approved if the break is intended."
  exit 1
fi

branch="stable/${major}"
# Only a confirmed-missing branch falls back to main: an API error must not widen approvals.
if ! lookup=$(gh api "repos/${GITHUB_REPOSITORY}/branches/${branch}" 2>&1 >/dev/null); then
  if ! grep -q "HTTP 404" <<<"$lookup"; then
    echo "::error::Cannot look up ${branch}: ${lookup}"
    exit 2
  fi
  branch="main"
fi
approved=$("$here/api-break-approvals.sh" "$tag" "$branch") || exit 2
if [ -z "$approved" ]; then
  echo "::error::Release blocked: the public API breaks compatibility with ${tag}. If intended, label the PR that made the change api-break-approved (or run this workflow manually with api_break_approved), then re-run."
  exit 1
fi

prs=$(cut -d' ' -f1 <<<"$approved" | xargs)
read -r latest_pr latest <<<"$approved"
if [ "$latest" != "$(git rev-parse HEAD)" ]; then
  dir=$(mktemp -d)
  trap 'git worktree remove --force "$dir" >/dev/null 2>&1; rm -rf "$dir"' EXIT
  git worktree add -q --detach "$dir" "$latest" || exit 2
  rc=0
  python3 "$here/api_compat.py" --baseline-root "$dir" || rc=$?
  if [ "$rc" -eq 1 ]; then
    echo "::error::Release blocked: breaking API changes were added after ${latest_pr} (${latest}), the newest approved PR. Label the PR that made them api-break-approved (or run this workflow manually with api_break_approved), then re-run."
    exit 1
  fi
  [ "$rc" -eq 0 ] || exit "$rc"
fi
echo "::warning::Breaking API changes approved with the api-break-approved label on: ${prs}"
