# AGENTS.md

> This repo follows the central Camunda AGENTS guidelines:
> https://raw.githubusercontent.com/camunda/.github/refs/heads/main/AGENTS.md
> The instructions below extend those and take precedence on conflict.

<!-- sdk-infra-delegation:start -->
> **Note:** This file also delegates to the shared SDK AGENTS.md. Read and apply it too.

**URL:**
https://raw.githubusercontent.com/camunda/sdk-infra/refs/heads/main/policies/AGENTS.md

It holds the conventions shared by all Camunda SDK repos (commits, bug fixes, tests, opening issues). Precedence on conflict: this file, then the shared SDK file, then the central Camunda file.
<!-- sdk-infra-delegation:end -->

## Role & boundary

This repo produces the **Rust SDK** for the Camunda 8 Orchestration Cluster REST API. It is a
sibling of the JS / Python / C# SDKs and follows the same pattern: a generated low-level
client wrapped by a hand-written ergonomic runtime. Target API version: **8.11** (`main`).

Upstream dependencies — fix at the source when they misbehave, not by working around them here:

- [`camunda-schema-bundler`](https://github.com/camunda/camunda-schema-bundler) — fetches and
  bundles the upstream OpenAPI spec.
- [`@openapitools/openapi-generator-cli`](https://github.com/OpenAPITools/openapi-generator) —
  generates `client/` (the `rust` generator, reqwest + serde, async).
- [`camunda/camunda`](https://github.com/camunda/camunda) — source of the OpenAPI spec.

## Path map

| Path | Ownership and intent |
| --- | --- |
| `src/runtime/` | Hand-written runtime. Modules: `config` (env resolution), `auth` (OAuth/Basic/None + disk token cache), `errors`, `client` (curated facade), `facade_generated` (**generated** full-surface facade — never hand-edit), `job_worker` (workers + lifecycle), `backpressure`, `retry` (transient HTTP retry), `eventual` (consistency polling), `tls` (mTLS), `logging` (tracing). Primary edit surface. |
| `src/lib.rs` | Public crate surface and re-exports. |
| `client/` | **Generated.** Produced by `make generate`. Never hand-edit. |
| `scripts/generate.sh` | Pipeline orchestrator: bundle → preprocess → generate → post-process → build. |
| `scripts/preprocess.py` | Derives the generator input spec from the bundle (rewrites `x-clearable` date-times the generator cannot type). |
| `scripts/postprocess.py` | Orchestrates the numbered post-processing hooks (`scripts/hooks/`). |
| `scripts/hooks/` | Numbered hooks: `01` Domain Type System, `02` semantic field types, `03` module-path/`Object` fixes, `04` regex dep, `05` lint silencing, `06` cleanup, `07` full-surface SDK facade, … `19` clearable date-times. Primary edit surface for fixing generated output. |
| `openapi-generator-config.yaml` | openapi-generator configuration. |
| `external-spec/bundled/` | Bundled spec (`rest-api.bundle.json`) + `spec-metadata.json`. A faithful copy of upstream; generator inputs. |
| `external-spec/generator/` | Transient generator input derived by `scripts/preprocess.py`. **Never commit** (gitignored). |
| `external-spec/upstream/` | Transient sparse clone of upstream. **Never commit** (gitignored). |
| `examples/` | Runnable examples. `readme.rs` hosts the region-tagged snippets injected into `README.md` (compiled by `make examples`). |
| `scripts/sync-readme-snippets.py` | Injects `examples/*.rs` region snippets into `README.md`; `--check` is the CI gate. |

## The Camunda Domain Type System (important)

The spec marks identifier schemas with `x-semantic-type` (`JobKey`, `ProcessInstanceKey`, …)
and defines `*ExactMatch` filter wrappers as `allOf: [<key>]`. `openapi-generator` handles
these **incorrectly** — it emits broken empty structs (`pub struct ScopeKey {}`) or drops the
model entirely, leaving dangling `models::JobKey` references.

`scripts/hooks/hook_01_domain_type_system.py` replaces them all with validated nominal
newtypes (`#[serde(transparent)] pub struct JobKey(String)`) carrying `try_new` /
`assume_exists` / `is_valid` and the `CamundaKey` trait. The generator also collapses plain
`type: string` semantic scalars (`ProcessDefinitionId`, `ElementId`, `TenantId`, …) to bare
`String` in field positions; `hook_02_semantic_field_types.py` rewrites those struct fields
and their `new()` params back to the generated newtypes. If you change how keys are
represented, change it there.

## Clearable date-times

The API clears some dates (`dueDate` / `followUpDate` on `JobResultCorrections` and the
user-task `Changeset`) with an empty string; `null` or omission preserves them. Upstream
marks them `x-clearable` (`ClearableDateTime`), which the generator turns into an empty
struct. `scripts/preprocess.py` rewrites them to nullable `date-time`s before generation
and `hook_19_clearable_date_time.py` keeps the `Option<Option<DateTime>>` field, sends
`Some(None)` as `""`, and adds `clear_<field>()` / `is_<field>_cleared()`. Both detect the
fields from `x-clearable` and from the older "empty String" description, and fail the run
if a detected field's generated code has an unexpected shape.

## Generation pipeline

```bash
make bundle      # re-bundle upstream spec (ref: $SPEC_REF, default main) + regenerate
make generate    # regenerate from the already-bundled spec
```

`scripts/generate.sh` runs:

1. (`--bundle`) `camunda-schema-bundler --ref main` → `external-spec/bundled/*`.
2. `scripts/preprocess.py` → `external-spec/generator/rest-api.json`, the spec
   `openapi-generator-config.yaml` reads. Fix a shape here only when the generator cannot
   emit usable code for it at all; otherwise fix the output in a hook.
3. `openapi-generator generate -c openapi-generator-config.yaml` → `client/`.
4. `scripts/postprocess.py` — runs the numbered hooks (`scripts/hooks/`): Domain Type
   System, semantic field types, generator-bug fixes, and the full-surface facade
   (`hook_07` parses `client/src/apis/*_api.rs` and emits `src/runtime/facade_generated.rs`,
   one `CamundaClient` method per operation routed through `guarded`). It parses the
   **unformatted** generator output, so it runs before step 5's `cargo fmt`.
5. `cargo fmt --all` (the client crate and the hook-emitted `src/runtime/*_generated.rs`),
   then `cargo build` on the client crate.

The hooks are **idempotent** and fix: missing/broken semantic keys, collapsed semantic
`String` fields, doubled
`models::models::` module paths, bare `Object` placeholders, and lint noise (it prepends
`#![allow(clippy::all)] #![allow(warnings)]` to the generated `client/src/lib.rs`).

## Build / test / lint

```bash
make build         # cargo build --workspace
make test          # cargo test --workspace  (unit + doctests)
make test-live     # live-cluster tests (#[ignore]d; needs a running cluster)
make examples      # cargo build --examples  (type-checks README snippet sources)
make lint          # cargo clippy --workspace --all-targets
make fmt           # cargo fmt --all
make sync-readme       # inject examples/*.rs region snippets into README.md
make sync-readme-check # CI gate: README in sync + every ```rust block backed by an example
make check         # full local CI gate (build, test, examples, lint, fmt-check, readme-sync)
```

CI (`.github/workflows/ci.yml`) runs the same gate (clippy with `-D warnings`) plus a
required generation-drift job that regenerates from upstream and fails on any diff. It uploads
the diff as the `generation-drift` artifact; `make bundle` (or applying that patch) and committing
the result clears it.

### Live-cluster tests

Tests that need a running cluster live in `tests/live_*.rs` and are marked
`#[ignore = "needs a live Camunda cluster; ..."]`, so `make test` and CI stay hermetic.
`make test-live` runs them against `CAMUNDA_REST_ADDRESS` (default
`http://localhost:8080`, auth from the usual env vars). Give every process id, job type and
element id a unique suffix — the cluster may be shared — and wait with the SDK's eventual
polling (`eventual_until`), never a sleep.

### README examples are compiled (no drift)

Every `rust` code block in `README.md` is injected from a region in `examples/readme.rs` by
`scripts/sync-readme-snippets.py`. Regions are delimited by `// region <Name>` /
`// endregion <Name>` and are type-checked via `make examples`. To change a documented
snippet: edit the region, run `make sync-readme`, commit both files. **Run `cargo fmt`
before `make sync-readme`** — rustfmt may reflow snippet source, which would otherwise make
the README and CI disagree.

### Ambient time is banned outside the clock adapter

`clippy.toml` bans `Instant::now`, `SystemTime::now`, `tokio::time::sleep`/`sleep_until`
and `std::thread::sleep`. Cadence must resolve through the injected `Clock`, or a test
clock controls nothing. `tokio::time::timeout` is deliberately *not* banned -- a timeout
is an I/O bound, not cadence.

The handful of legitimate uses carry a narrow `#[allow(clippy::disallowed_methods)]` on
the specific statement or impl block, never a module-level allow, so new ambient-time
calls in those same files are still caught. If you need a new exception, scope it the
same way and say why.

### Always-green policy

The **hand-written** crate (`src/`) must be warning- and clippy-clean. Generated lint noise in
`client/` is suppressed at the crate root by the post-processor — do not silence lints in
`src/` to make a build pass.

## Separate generator changes from regenerated output

When a change modifies the generator (post-processor, config, scripts) **and** that change
alters `client/`, split into two commits:

1. Generator change only (scripts / config). No `client/` changes.
2. `chore(gen): regenerate client for <summary>` — the regenerated `client/` output.

This keeps cherry-picks clean and reviews readable, and preserves `git blame` on both surfaces.

## Backporting to stable branches

`.github/workflows/backport.yml` calls camunda/sdk-infra's shared `sdk-backport.yml`
(`korthout/backport-action`, running as `camunda-sdk-automation`; the same flow as the other
SDKs). It is configured by `.github/backport.yml` and `.github/actions/codegen-setup` (Java
for openapi-generator), both read from the **target** branch, so keep them on `main` and on
every live `stable/N`. It opens cherry-pick PRs onto `stable/*` branches:

- Add the label `backport stable/<N>` to a PR. The backport PR opens when the PR merges,
  or right away if it is already merged.
- Or comment `/backport` on a merged PR to re-run it for its labels. PRs from forks can
  only be backported this way.

Backport hand-written runtime, infra and generator changes (`src/`, `scripts/`, `tests/`)
when the affected code exists on the stable branch. Never backport generated output on its
own (e.g. a regeneration for upstream spec drift); each branch regenerates from its own
upstream spec.

After the cherry-pick, the workflow adds a `chore(gen): regenerate for backport of #N`
commit. The derived files (`generated-paths`: `external-spec/`, `client/`,
`src/runtime/facade_generated.rs`, `src/runtime/present_when_generated.rs`) are restored
from the target branch and regenerated with `./scripts/generate.sh && cargo test
--workspace --no-run`. PRs are squash-merged, so the cherry-pick carries `main`'s copies,
but a stable branch must build from its own spec (`stable/10` builds camunda
`stable/8.10`). This also clears cherry-pick conflicts in those files. When the derived
paths change (e.g. a new hook writes another `*_generated.rs`), update `generated-paths` on
every branch.

If the cherry-pick conflicts, the action opens a **draft** PR with the conflicts committed.
A draft always means a conflict, even without conflict markers: if the source changes a file
the target lacks, `main`'s whole file is committed. Resolve it on the backport branch, then
comment `/backport` on the source PR to regenerate. If fix B builds on fix A, merge A's
backport first, then rebuild B's branch from the updated target (`git switch -C
backport-<B>-to-stable/N origin/stable/N && git cherry-pick -x <B's merge commit>`, push
with `--force-with-lease`, comment `/backport`).

If regenerating fails, the bot comments on the backport PR. If the change depends on spec
the stable branch doesn't have, it can't be backported as-is: close the backport PR, or
reduce it to what applies to the stable spec.

## Versioning

The SDK major tracks the Camunda server minor (server 8.10 → SDK 10.x): `stable/10` is
10.x, `main` is the next line. Both crates share one version, set in `Cargo.toml`,
`client/Cargo.toml` (from `packageVersion` in `openapi-generator-config.yaml`) and the
SDK's path dependency on the client. A published version can only be yanked.

### Releases are automatic

Releases are cut by semantic-release (`.github/workflows/version.yml`, `release.config.cjs`
on top of the shared base config in `@camunda8/sdk-infra`) from the Conventional Commits
merged to a release branch:

- `stable/N` releases `N.x.y`: `fix`/`perf`/`revert` -> patch, `feat` -> minor.
- `main` releases prereleases of the next line, `<N+1>.0.0-alpha.M`.

`scripts/verify-release-major.sh` refuses any other version, so a `BREAKING CHANGE` on
`stable/N` stops the release instead of bumping the major. A release runs
`scripts/set-release-version.sh` (both crates, the client path dependency, `packageVersion`
and `Cargo.lock`), commits `chore(release): X.Y.Z` with a `CHANGELOG.md` entry, pushes the
`vX.Y.Z` tag and creates the GitHub Release. The tag triggers `release.yml`, which publishes
to crates.io. **Never bump versions or push `v*` tags by hand.**

The workflow runs as the `camunda-sdk-automation` app, which bypasses the branch rulesets,
and reads the repo variable `CAMUNDA_SDK_CURRENT_STABLE_MAJOR` (the newest `stable/N`).
When a new line is branched, update the variable and seed `main` with a
`v<N+1>.0.0-alpha.0` tag on a commit made **after** the branch point, with the git note
`refs/notes/semantic-release-v<N+1>.0.0-alpha.0` = `{"channels":["alpha"]}`. Without a
main-only tag, the stable branch's version range is empty and its releases fail.
`release.yml` ignores `*.0.0-alpha.0` tags, so the seed is never published.

Preview the next release locally with
`GITHUB_REF_NAME=<branch> CAMUNDA_SDK_CURRENT_STABLE_MAJOR=<N> npx semantic-release --dry-run --no-ci`
(needs a token that can read the repo in `GITHUB_TOKEN`).

### API compatibility on stable branches

`stable/N` regenerates from upstream `stable/8.x`, so spec drift can break the public API
of an `N.x` patch. The `API compatibility` check (`.github/workflows/api-compat.yml`) runs
on every PR to `stable/**` and compares it with its base branch, so a PR answers only for
its own changes. Once an `N.x` release exists, `release.yml` compares the version it is
about to publish with the newest non-yanked, non-prerelease `N.x.y` on crates.io. Both check
both crates (the client is public through `client`, `apis` and `models`) using
`scripts/api_compat.py`: pinned cargo-semver-checks plus a rustdoc JSON diff of every
public field/variant/fn/method signature, compared as written, which catches `T` ->
`Option<T>` (cargo-semver-checks does not). An equivalent respelling, such as a renamed
generic, is reported too. Run it locally with `make api-compat`; test the checker with
`make test-api-compat`.

A break is fixed by making the change compatible. If it is intended, a maintainer adds the
`api-break-approved` label to the PR. The release gate (`scripts/api-compat-release-gate.sh`)
accepts the label only on a PR merged into the release line since the baseline, and only
for the API as of that PR's merge: a break added afterwards needs its own approval (or a
`workflow_dispatch` run with `api_break_approved: true`).

## Commit messages

Conventional Commits. Use `fix` only for user-facing bug fixes (triggers a release); use
`chore` for review-comment fix-ups and regeneration commits.
