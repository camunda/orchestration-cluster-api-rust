# Camunda Orchestration Cluster API — Rust SDK

## Status: Technical Preview

This is a technical preview of the Rust SDK for the Camunda 8 Orchestration Cluster API, provided for evaluation and feedback. We do not intend to make breaking changes to the application integration surface, but do not guarantee that we will not. It will become fully supported with an SLA in a future release.  

Ergonomic Rust SDK for the [Camunda 8 Orchestration Cluster REST API](https://docs.camunda.io/docs/apis-tools/orchestration-cluster-api-rest/orchestration-cluster-api-rest-overview/).

This SDK follows the same architecture as the official
[JavaScript](https://github.com/camunda/orchestration-cluster-api-js),
[Python](https://github.com/camunda/orchestration-cluster-api-python), and
[C#](https://github.com/camunda/orchestration-cluster-api-csharp) SDKs:

- A **generated low-level client** (`camunda-orchestration-api-client`) produced from the
  upstream OpenAPI spec (sourced and bundled with
  [`camunda-schema-bundler`](https://github.com/camunda/camunda-schema-bundler)).
- A **hand-written runtime** (`camunda-orchestration-sdk`) on top: environment-driven
  configuration, authentication, typed errors, and **job workers**.
- The **Camunda Domain Type System**: semantic keys such as `JobKey` and
  `ProcessInstanceKey` are nominal newtypes with validation, not bare strings.

Target API version: **8.10** (`main`).

## Workspace layout

```
.
├── Cargo.toml                       # workspace + `camunda-orchestration-sdk` crate
├── src/                             # hand-written runtime (NEVER generated)
│   ├── lib.rs
│   └── runtime/{config,auth,errors,client,job_worker}.rs
├── client/                          # GENERATED low-level crate — never hand-edit
│   ├── src/apis/                    #   one module per API tag (reqwest, async)
│   └── src/models/                  #   request/response models + camunda_keys.rs (domain types)
├── external-spec/bundled/           # bundled OpenAPI spec + metadata (generator input)
├── scripts/
│   ├── generate.sh                  # bundle → openapi-generator → post-process → build
│   ├── postprocess.py               # orchestrates the numbered post-processing hooks
│   └── hooks/                       # numbered hooks: Domain Type System, semantic fields, fixups
├── openapi-generator-config.yaml
├── examples/
└── Makefile
```

## Installation

This SDK is a Cargo workspace. To use it from another crate while it is unpublished:

```toml
[dependencies]
camunda-orchestration-sdk = { path = "../orchestration-cluster-api-rust" }
tokio = { version = "1", features = ["full"] }
```

The async API requires a Tokio runtime.

## Quick start

<!-- snippet-source: examples/readme.rs | regions: QuickStart -->
```rust
use camunda_orchestration_sdk::CamundaClient;

// Reads CAMUNDA_REST_ADDRESS, CAMUNDA_AUTH_STRATEGY, CAMUNDA_CLIENT_ID, ... from the env.
let client = CamundaClient::from_env()?;

let topology = client.topology().await?;
println!("Gateway version: {}", topology.gateway_version);
```

Programmatic configuration overrides take precedence over the environment:

<!-- snippet-source: examples/readme.rs | regions: Overrides -->
```rust
use camunda_orchestration_sdk::{CamundaClient, CamundaOptions};

let client = CamundaClient::new(
    CamundaOptions::new()
        .with("CAMUNDA_REST_ADDRESS", "https://my-cluster.camunda.io")
        .with("CAMUNDA_AUTH_STRATEGY", "OAUTH")
        .with("CAMUNDA_CLIENT_ID", "my-client-id")
        .with("CAMUNDA_CLIENT_SECRET", "my-secret")
        .with(
            "CAMUNDA_OAUTH_URL",
            "https://login.cloud.camunda.io/oauth/token",
        )
        .with("CAMUNDA_TOKEN_AUDIENCE", "zeebe.camunda.io"),
)?;
```

## Authentication

The strategy is chosen by `CAMUNDA_AUTH_STRATEGY` (or inferred from the presence of OAuth
credentials):

| Strategy | Required configuration |
| --- | --- |
| `OAUTH` | `CAMUNDA_CLIENT_ID`, `CAMUNDA_CLIENT_SECRET`, `CAMUNDA_OAUTH_URL`, optional `CAMUNDA_TOKEN_AUDIENCE`, `CAMUNDA_OAUTH_SCOPE` |
| `BASIC` | `CAMUNDA_BASIC_AUTH_USERNAME`, `CAMUNDA_BASIC_AUTH_PASSWORD` |
| `NONE`  | — (local development) |

OAuth uses the client-credentials grant. Tokens are cached in memory and refreshed shortly
before expiry.

## Configuration reference

| Variable | Purpose |
| --- | --- |
| `CAMUNDA_REST_ADDRESS` | Base cluster address. `/v2` is appended automatically. |
| `ZEEBE_REST_ADDRESS` | Alias for `CAMUNDA_REST_ADDRESS`. |
| `CAMUNDA_AUTH_STRATEGY` | `OAUTH` \| `BASIC` \| `NONE`. |
| `CAMUNDA_CLIENT_ID` / `CAMUNDA_CLIENT_SECRET` | OAuth client credentials. |
| `CAMUNDA_OAUTH_URL` | OAuth token endpoint. |
| `CAMUNDA_TOKEN_AUDIENCE` | OAuth `audience` parameter. |
| `CAMUNDA_OAUTH_SCOPE` | OAuth `scope` parameter. |
| `CAMUNDA_BASIC_AUTH_USERNAME` / `CAMUNDA_BASIC_AUTH_PASSWORD` | Basic-auth credentials. |
| `CAMUNDA_DEFAULT_TENANT_ID` | Default tenant id (alias `CAMUNDA_TENANT_ID`). Injected into deploys, instance creation, messages, signals, decisions, and worker activation when none is set. |
| `CAMUNDA_SDK_BACKPRESSURE_PROFILE` | Adaptive backpressure profile: `BALANCED` (default, adaptive gating) or `LEGACY` (observe-only, no gating). |
| `CAMUNDA_OAUTH_CACHE_DIR` | Directory for the cross-process OAuth token cache. Unset disables disk caching (in-memory only). |
| `CAMUNDA_SDK_LOG_LEVEL` | SDK log level for [`CamundaClient::init_logging`]: `OFF` \| `ERROR` \| `WARN` \| `INFO` (default) \| `DEBUG` \| `TRACE`. |
| `CAMUNDA_SDK_EVENTUAL_POLL_DEFAULT_MS` | Default timeout for [`CamundaClient::eventual`] consistency polling (default `10000`). |
| `CAMUNDA_SDK_HTTP_RETRY_MAX_ATTEMPTS` | Max attempts for transient-error retry of initiating operations (default `4`; `1` disables retry). |
| `CAMUNDA_SDK_HTTP_RETRY_BASE_DELAY_MS` / `CAMUNDA_SDK_HTTP_RETRY_MAX_DELAY_MS` | Full-jitter backoff bounds for HTTP retry (defaults `250` / `5000`). |
| `CAMUNDA_MTLS_CERT` / `CAMUNDA_MTLS_CERT_PATH` | Client certificate (inline PEM or file path) for mutual TLS. |
| `CAMUNDA_MTLS_KEY` / `CAMUNDA_MTLS_KEY_PATH` | Client private key (inline PEM or file path) for mutual TLS. |
| `CAMUNDA_MTLS_CA` / `CAMUNDA_MTLS_CA_PATH` | Additional CA root (inline PEM or file path) to trust. |
| `CAMUNDA_MTLS_KEY_PASSPHRASE` | Passphrase for an encrypted client key (not supported by the default `native-tls` backend; errors clearly if set). |
| `CAMUNDA_WORKER_NAME` | Default worker name for [`CamundaClient::worker_config`]. |
| `CAMUNDA_WORKER_MAX_CONCURRENT_JOBS` | Default max concurrent jobs per worker. |
| `CAMUNDA_WORKER_TIMEOUT` / `CAMUNDA_WORKER_REQUEST_TIMEOUT` | Default job-activation and long-poll timeouts (ms). |
| `CAMUNDA_WORKER_STARTUP_JITTER_MAX_SECONDS` | Max random startup delay before a worker's first poll, to spread activation stampedes. |

## Backpressure

The client includes an adaptive global backpressure controller, mirroring the JS/Python
SDKs. Initiating operations (`create_process_instance`, `activate_jobs`, `deploy_resources`,
`topology`) pass through an AIMD-style concurrency gate that reacts to cluster backpressure
signals (HTTP `429` / `503` / `RESOURCE_EXHAUSTED`):

- Starts **unlimited**; on the first backpressure signal it boots to an initial permit cap
  and shrinks multiplicatively (soft ×0.70, severe ×0.50 after 3 consecutive signals) down
  to a floor of 1, with an escalating 25 ms → 2 s backoff while stuck at the floor.
- Recovers when quiet: additive growth while unhealthy, then multiplicative growth while
  healthy, returning to unlimited after a sustained-healthy period.
- **Drain operations** (`complete_job`, `fail_job`, `throw_job_error`) bypass the gate so
  in-flight work always drains, even while new load is being shed.

Set `CAMUNDA_SDK_BACKPRESSURE_PROFILE=LEGACY` to observe signals without gating. Inspect the
live state via [`CamundaClient::backpressure_state`].

## Reliability & convenience features

The runtime mirrors the JS/Python/C# SDKs:

- **Transient HTTP retry** — initiating operations retry `429`/`502`/`503`/`504` and network
  errors with full-jitter backoff (`CAMUNDA_SDK_HTTP_RETRY_*`). Drain operations are never
  retried blindly.
- **Eventual-consistency polling** — `client.eventual(opts, op)` / `eventual_until(opts, op,
  predicate)` retry `404` reads (the symptom of replication lag) until consistent or the
  window elapses.
- **Mutual TLS** — client cert/key/CA from `CAMUNDA_MTLS_*` (inline PEM or file path).
- **OAuth disk token cache** — set `CAMUNDA_OAUTH_CACHE_DIR` to share tokens across processes
  (atomic write, namespaced per client/audience).
- **Default-tenant injection** — `CAMUNDA_DEFAULT_TENANT_ID` is applied automatically wherever
  a tenant is accepted.
- **Configurable logging** — `client.init_logging()` installs a `tracing` subscriber filtered
  to `CAMUNDA_SDK_LOG_LEVEL`.
- **Facade convenience methods** — `cancel_process_instance`, `get_process_instance`,
  `publish_message`, `correlate_message`, `broadcast_signal`, `evaluate_decision`,
  `search_variables` / `search_variables_as::<T>()`.
- **Worker lifecycle** — `client.spawn_worker(..)` registers managed workers;
  `running_workers()` lists them and `stop_all_workers().await` drains and stops them all
  gracefully. Per-worker control via the [`JobWorkerHandle`] returned from
  [`JobWorker::spawn`].

## Job workers

<!-- snippet-source: examples/readme.rs | regions: JobWorker -->
```rust
use camunda_orchestration_sdk::{JobAction, JobWorkerConfig};

let worker = client.create_job_worker(
    JobWorkerConfig::new("payment-service")
        .max_jobs_to_activate(20)
        .worker_name("payment-worker"),
);

worker
    .run(|job| async move {
        println!("handling job {}", job.key());
        JobAction::complete_with(serde_json::json!({ "paid": true }))
    })
    .await?;
```

A handler returns a `JobAction`:

- `JobAction::complete()` / `JobAction::complete_with(vars)` — complete the job.
- `JobAction::fail("message")` — fail the job (retries decremented by the engine).
- `JobAction::error("ERROR_CODE")` — throw a catchable BPMN error.
- `JobAction::leave()` — take no action; the job remains activated until timeout.

The `Job` exposes `key()`, `job_type()`, `process_instance_key()`, `variables()`, and
`variables_as::<T>()` for typed deserialization.

For managed lifecycle, register workers on the client and stop them all gracefully:

<!-- snippet-source: examples/readme.rs | regions: GracefulShutdown -->
```rust
// Spawn managed workers; the client retains them in its registry.
client.spawn_worker(client.worker_config("payment-service"), |job| async move {
    JobAction::complete_with(serde_json::json!({ "paid": true }))
});

// ... later, on shutdown: drain in-flight jobs and stop every worker gracefully.
client.stop_all_workers().await?;
```

## Eventual consistency

Camunda's read APIs are eventually consistent. Wrap a read in `eventual` to poll through
replication lag, transparently retrying `404`s:

<!-- snippet-source: examples/readme.rs | regions: EventualConsistency -->
```rust
use camunda_orchestration_sdk::ConsistencyOptions;

// Reads are eventually consistent: poll until the instance is visible, retrying 404s.
let instance = client
    .eventual(ConsistencyOptions::default(), || {
        let client = client.clone();
        let key = process_instance_key.clone();
        async move { client.get_process_instance(&key).await }
    })
    .await?;
```

## The Camunda Domain Type System

Camunda's spec marks identifier schemas with `x-semantic-type` (e.g. `JobKey`,
`ProcessInstanceKey`, and `*ExactMatch` filter variants). The SDK models each as a nominal
newtype over its string value, serialized transparently:

<!-- snippet-source: examples/readme.rs | regions: SemanticKeys -->
```rust
use camunda_orchestration_sdk::models::{CamundaKey, JobKey};

let key = JobKey::try_new("2251799813653498")?; // validates pattern & length
assert_eq!(key.value(), "2251799813653498");
// Side-load without validation:
let loose = JobKey::assume_exists("123");
```

These types are **generated by post-processing**, because `openapi-generator` does not handle
the semantic-key schemas correctly (it emits broken empty structs or drops them).

## Using operations not yet wrapped by the facade

The ergonomic facade wraps common operations (`topology`, `create_process_instance`,
`deploy_resources`, job operations). For anything else, build a generated `Configuration`
and call the generated API directly:

<!-- snippet-source: examples/readme.rs | regions: EscapeHatch -->
```rust
use camunda_orchestration_sdk::client::apis::authentication_api;
use camunda_orchestration_sdk::CamundaClient;

let client = CamundaClient::from_env()?;
let cfg = client.configuration().await?; // base URL + auth applied
let me = authentication_api::get_authentication(&cfg).await?;
```

## Regenerating the client

Requirements: Rust toolchain, Java 17+ (for openapi-generator), Node.js (for
`camunda-schema-bundler` and `@openapitools/openapi-generator-cli`, both run via `npx`).

```bash
make bundle      # re-fetch + bundle the upstream spec (ref: main), then regenerate
make generate    # regenerate from the already-bundled spec
make build
make test
make lint
```

The generation pipeline (`scripts/generate.sh`):

1. (`--bundle` only) `camunda-schema-bundler` fetches and bundles the upstream OpenAPI spec.
2. `openapi-generator` produces the `client/` crate (reqwest + serde, async).
3. `scripts/postprocess.py` runs the numbered hooks under `scripts/hooks/`: generate the
   Domain Type System, apply semantic newtypes to fields the generator collapsed to
   `String`, and fix known generator output bugs (doubled `models::models::` paths, bare
   `Object` placeholders, lint noise).

**`client/` is generated and must never be hand-edited.** Fix generator output in the
post-processor instead.

## Documentation examples

Every `rust` code block in this README is injected from a **compilable** example in
`examples/readme.rs`, so the documentation can never drift from the real API.

- Snippets live between `// region <Name>` / `// endregion <Name>` markers in
  `examples/readme.rs` (or any `examples/*.rs`) and are type-checked by `make examples`
  (`cargo build --examples`).
- README code blocks are tagged with `<!-- snippet-source: examples/readme.rs | regions: <Name> -->`.
- `make sync-readme` injects the current example code into the README;
  `make sync-readme-check` (run in CI) fails if the README is out of date or if a `rust`
  block is not backed by an example.

To change a documented example, edit the region in `examples/readme.rs`, then run
`make sync-readme`. The full local gate is `make check`.

## License

The Camunda OpenAPI specification is subject to the
[Camunda License Version 1.0](https://github.com/camunda/camunda/blob/main/licenses/CAMUNDA-LICENSE-1.0.txt).
