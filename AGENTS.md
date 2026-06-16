# AGENTS.md

> This repo follows the central Camunda AGENTS guidelines:
> https://raw.githubusercontent.com/camunda/.github/refs/heads/main/AGENTS.md
> The instructions below extend those and take precedence on conflict.

## Role & boundary

This repo produces the **Rust SDK** for the Camunda 8 Orchestration Cluster REST API. It is a
sibling of the JS / Python / C# SDKs and follows the same pattern: a generated low-level
client wrapped by a hand-written ergonomic runtime. Target API version: **8.10** (`main`).

Upstream dependencies — fix at the source when they misbehave, not by working around them here:

- [`camunda-schema-bundler`](https://github.com/camunda/camunda-schema-bundler) — fetches and
  bundles the upstream OpenAPI spec.
- [`@openapitools/openapi-generator-cli`](https://github.com/OpenAPITools/openapi-generator) —
  generates `client/` (the `rust` generator, reqwest + serde, async).
- [`camunda/camunda`](https://github.com/camunda/camunda) — source of the OpenAPI spec.

## Path map

| Path | Ownership and intent |
| --- | --- |
| `src/runtime/` | Hand-written runtime. Modules: `config` (env resolution), `auth` (OAuth/Basic/None + disk token cache), `errors`, `client` (facade), `job_worker` (workers + lifecycle), `backpressure`, `retry` (transient HTTP retry), `eventual` (consistency polling), `tls` (mTLS), `logging` (tracing). Primary edit surface. |
| `src/lib.rs` | Public crate surface and re-exports. |
| `client/` | **Generated.** Produced by `make generate`. Never hand-edit. |
| `scripts/generate.sh` | Pipeline orchestrator: bundle → generate → post-process → build. |
| `scripts/postprocess.py` | Orchestrates the numbered post-processing hooks (`scripts/hooks/`). |
| `scripts/hooks/` | Numbered hooks: `01` Domain Type System, `02` semantic field types, `03` module-path/`Object` fixes, `04` regex dep, `05` lint silencing, `06` cleanup. Primary edit surface for fixing generated output. |
| `openapi-generator-config.yaml` | openapi-generator configuration. |
| `external-spec/bundled/` | Bundled spec (`rest-api.bundle.json`) + `spec-metadata.json`. Generator inputs. |
| `external-spec/upstream/` | Transient sparse clone of upstream. **Never commit** (gitignored). |
| `examples/` | Runnable examples. |

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

## Generation pipeline

```bash
make bundle      # re-bundle upstream spec (ref: $SPEC_REF, default main) + regenerate
make generate    # regenerate from the already-bundled spec
```

`scripts/generate.sh` runs:

1. (`--bundle`) `camunda-schema-bundler --ref main` → `external-spec/bundled/*`.
2. `openapi-generator generate -c openapi-generator-config.yaml` → `client/`.
3. `scripts/postprocess.py` — runs the numbered hooks (`scripts/hooks/`): Domain Type
   System, semantic field types, and generator-bug fixes.
4. `cargo fmt` + `cargo build` on the client crate.

The hooks are **idempotent** and fix: missing/broken semantic keys, collapsed semantic
`String` fields, doubled
`models::models::` module paths, bare `Object` placeholders, and lint noise (it prepends
`#![allow(clippy::all)] #![allow(warnings)]` to the generated `client/src/lib.rs`).

## Build / test / lint

```bash
make build   # cargo build --workspace
make test    # cargo test --workspace  (unit + doctests)
make lint    # cargo clippy --workspace
make fmt     # cargo fmt --all
```

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

## Commit messages

Conventional Commits. Use `fix` only for user-facing bug fixes (triggers a release); use
`chore` for review-comment fix-ups and regeneration commits.
