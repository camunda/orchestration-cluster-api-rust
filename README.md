# Camunda Orchestration Cluster API — Rust SDK

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
│   └── postprocess_domain_types.py  # generates the Domain Type System; fixes generator bugs
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

```rust
use camunda_orchestration_sdk::CamundaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Reads CAMUNDA_REST_ADDRESS, CAMUNDA_AUTH_STRATEGY, CAMUNDA_CLIENT_ID, ... from the env.
    let client = CamundaClient::from_env()?;

    let topology = client.topology().await?;
    println!("Gateway version: {}", topology.gateway_version);
    Ok(())
}
```

Programmatic configuration overrides take precedence over the environment:

```rust
use camunda_orchestration_sdk::{CamundaClient, CamundaOptions};

let client = CamundaClient::new(
    CamundaOptions::new()
        .with("CAMUNDA_REST_ADDRESS", "https://my-cluster.camunda.io")
        .with("CAMUNDA_AUTH_STRATEGY", "OAUTH")
        .with("CAMUNDA_CLIENT_ID", "my-client-id")
        .with("CAMUNDA_CLIENT_SECRET", "my-secret")
        .with("CAMUNDA_OAUTH_URL", "https://login.cloud.camunda.io/oauth/token")
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
| `CAMUNDA_DEFAULT_TENANT_ID` | Default tenant id (alias `CAMUNDA_TENANT_ID`). |
| `CAMUNDA_SDK_BACKPRESSURE_PROFILE` | Adaptive backpressure profile: `BALANCED` (default, adaptive gating) or `LEGACY` (observe-only, no gating). |

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

## Job workers

```rust
use camunda_orchestration_sdk::{CamundaClient, JobAction, JobWorkerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CamundaClient::from_env()?;

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
    Ok(())
}
```

A handler returns a `JobAction`:

- `JobAction::complete()` / `JobAction::complete_with(vars)` — complete the job.
- `JobAction::fail("message")` — fail the job (retries decremented by the engine).
- `JobAction::error("ERROR_CODE")` — throw a catchable BPMN error.
- `JobAction::leave()` — take no action; the job remains activated until timeout.

The `Job` exposes `key()`, `job_type()`, `process_instance_key()`, `variables()`, and
`variables_as::<T>()` for typed deserialization.

## The Camunda Domain Type System

Camunda's spec marks identifier schemas with `x-semantic-type` (e.g. `JobKey`,
`ProcessInstanceKey`, and `*ExactMatch` filter variants). The SDK models each as a nominal
newtype over its string value, serialized transparently:

```rust
use camunda_orchestration_sdk::models::{JobKey, CamundaKey};

let key = JobKey::try_new("2251799813653498")?; // validates pattern & length
assert_eq!(key.value(), "2251799813653498");
let loose = JobKey::assume_exists("123");        // side-load without validation
# Ok::<(), Box<dyn std::error::Error>>(())
```

These types are **generated by post-processing**, because `openapi-generator` does not handle
the semantic-key schemas correctly (it emits broken empty structs or drops them).

## Using operations not yet wrapped by the facade

The ergonomic facade wraps common operations (`topology`, `create_process_instance`,
`deploy_resources`, job operations). For anything else, build a generated `Configuration`
and call the generated API directly:

```rust
use camunda_orchestration_sdk::CamundaClient;
use camunda_orchestration_sdk::client::apis::authentication_api;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let client = CamundaClient::from_env()?;
let cfg = client.configuration().await?; // base URL + auth applied
let me = authentication_api::get_authentication(&cfg).await?;
# let _ = me;
# Ok(())
# }
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
3. `scripts/postprocess_domain_types.py` generates the Domain Type System and fixes known
   generator output bugs (doubled `models::models::` paths, bare `Object` placeholders,
   lint noise).

**`client/` is generated and must never be hand-edited.** Fix generator output in the
post-processor instead.

## License

The Camunda OpenAPI specification is subject to the
[Camunda License Version 1.0](https://github.com/camunda/camunda/blob/main/licenses/CAMUNDA-LICENSE-1.0.txt).
