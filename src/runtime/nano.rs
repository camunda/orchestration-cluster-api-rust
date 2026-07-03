//! Nano-aware Falcon transport.
//!
//! [nanobpmn](https://github.com/jwulf/nano-bpm) is an API/behaviour superset of the
//! Camunda 8 Orchestration Cluster. In addition to the REST API this SDK targets, a
//! nanobpmn gateway exposes a persistent **Falcon** WebSocket that adds
//! graceful, credit-metered flow control the REST path lacks:
//!
//! * process creation is gated on a per-connection **submission-credit** window, so a
//!   flood of creates *queues* on the client instead of being shed with `503`s; and
//! * job delivery is **pushed** to subscribed workers (credit-replenished on
//!   completion) rather than long-polled, removing the thundering-herd cost of many
//!   over-provisioned REST workers.
//!
//! This module is a thin, opt-in upgrade. A gateway is probed **once** per client via
//! `GET /v2/topology`; the response carries a `nano` object only on a nanobpmn gateway
//! ([`detect`]). Against stock Camunda the probe finds no `nano` field and the SDK
//! stays on its byte-identical REST path. The behaviour is gated by
//! `CAMUNDA_NANO_FALCON` (default on; set `off`/`false`/`0` to force pure REST).
//!
//! Only plaintext `ws://` is supported (local-cluster demos / non-TLS gateways); a
//! `wss://`-derived URL falls back to REST.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Map, Value};
use tokio::sync::{mpsc, oneshot, Notify};
use tokio_tungstenite::tungstenite::Message;

use camunda_orchestration_api_client::models;

use super::errors::{CamundaError, Result};

/// Default falcon path advertised by a nanobpmn gateway.
const DEFAULT_FALCON_PATH: &str = "/falcon";

/// How long to wait for a create `CommandResult` before giving up.
const CREATE_ACK_TIMEOUT: Duration = Duration::from_secs(30);

/// Detected nanobpmn capabilities, derived from `GET /v2/topology`.
#[derive(Debug, Clone)]
pub struct NanoCaps {
    /// Falcon WebSocket URLs for every node in the cluster (the failover
    /// directory). Single-element for a single-node gateway. The transport picks one at
    /// random on connect and fails over to another on disconnect.
    pub endpoints: Vec<String>,
}

/// Returns `true` unless `CAMUNDA_NANO_FALCON` is explicitly disabled.
pub fn falcon_enabled() -> bool {
    match std::env::var("CAMUNDA_NANO_FALCON") {
        Ok(v) => !matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "0" | "off" | "false" | "no"
        ),
        Err(_) => true,
    }
}

/// Probe a gateway's `/v2/topology` for the `nano` advertisement.
///
/// Returns `None` for stock Camunda (no `nano` field), when the Falcon is
/// disabled by config, when the gateway is not reachable over plaintext `ws://`, or on
/// any error — in every case the caller falls back to REST, so detection never fails a
/// request.
pub async fn detect(rest_address: &str, http: &reqwest::Client) -> Option<NanoCaps> {
    if !falcon_enabled() {
        return None;
    }
    // Plaintext ws only: a TLS gateway can't be upgraded here.
    if rest_address.trim_start().starts_with("https://") {
        return None;
    }
    let url = format!("{}/topology", rest_address.trim_end_matches('/'));
    let resp = http.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: Value = resp.json().await.ok()?;
    let nano = body.get("nano")?;
    if nano.is_null() {
        return None;
    }
    let falcon_path = nano
        .get("falconPath")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_FALCON_PATH)
        .to_string();
    let endpoints = endpoints_from_topology(rest_address, &falcon_path, &body);
    Some(NanoCaps { endpoints })
}

/// Build the falcon failover directory from a `/v2/topology` body.
///
/// Every `brokers[].host:port` becomes a `ws://host:port<path>` endpoint. A broker may
/// report itself as `0.0.0.0` (the self placeholder); that host is replaced with the host
/// we actually reached the gateway on, so the directory is dialable from the client. The
/// address we connected to is always included as a fallback, and the list is de-duplicated.
/// A single-node gateway yields a one-element directory (today's behaviour).
fn endpoints_from_topology(rest_address: &str, path: &str, body: &Value) -> Vec<String> {
    let connect_host = host_of(rest_address);
    let mut out: Vec<String> = Vec::new();
    let mut push = |url: String| {
        if !out.contains(&url) {
            out.push(url);
        }
    };

    // The address the client was configured with is always a valid entry.
    push(ws_url(rest_address, path));

    if let Some(brokers) = body.get("brokers").and_then(Value::as_array) {
        for b in brokers {
            let host = b.get("host").and_then(Value::as_str).unwrap_or("");
            let port = b.get("port").and_then(Value::as_i64).unwrap_or(0);
            if port == 0 {
                continue;
            }
            // Replace the self/unspecified placeholder with the reachable host.
            let host = if host.is_empty() || host == "0.0.0.0" || host == "::" || host == "[::]" {
                connect_host.as_deref().unwrap_or("127.0.0.1")
            } else {
                host
            };
            push(format!("ws://{host}:{port}{path}"));
        }
    }

    out
}

/// Extract the bare host of a `scheme://host:port[/...]` address.
fn host_of(address: &str) -> Option<String> {
    let after = address
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(address);
    let authority = after.split(['/', '?']).next().unwrap_or(after);
    // Strip any :port (IPv6 literals are out of scope for the local-cluster demo).
    let host = authority
        .rsplit_once(':')
        .map(|(h, _)| h)
        .unwrap_or(authority);
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

/// Derive the falcon WebSocket URL from the REST address.
///
/// `rest_address` is normalised to `<scheme>://host:port/v2`; the Falcon lives
/// at `ws://host:port<path>` (the `/v2` prefix is stripped and `http`→`ws`).
pub fn ws_url(rest_address: &str, path: &str) -> String {
    let trimmed = rest_address.trim_end_matches('/');
    let origin = trimmed.strip_suffix("/v2").unwrap_or(trimmed);
    let ws_origin = if let Some(rest) = origin.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = origin.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        origin.to_string()
    };
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    format!("{ws_origin}{path}")
}

/// The writer sender and decoded-frame receiver produced by [`dial`].
type DialPair = (
    mpsc::UnboundedSender<Message>,
    mpsc::UnboundedReceiver<Value>,
);

/// Open a falcon WebSocket and split it into a writer task (fed by the returned
/// sender) and a reader task that decodes server frames into the returned channel.
///
/// Heartbeats and Pings are answered inline by the reader (so the gateway doesn't reap an
/// idle socket); every other text frame is forwarded as parsed JSON. When the socket
/// closes or errors, the frame channel ends (`recv()` yields `None`), which the
/// [`SupervisedLink`] supervisor treats as a disconnect.
async fn dial(url: &str) -> Result<DialPair> {
    let (ws, _resp) = tokio_tungstenite::connect_async(url)
        .await
        .map_err(|e| CamundaError::Config(format!("nano falcon connect failed: {e}")))?;
    let (mut sink, mut stream) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    let (frame_tx, frame_rx) = mpsc::unbounded_channel::<Value>();

    // Writer task: forward queued frames to the socket.
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
        let _ = sink.close().await;
    });

    // Reader task: decode text frames, auto-answer heartbeats/pings, forward the rest.
    let writer = tx.clone();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(value) = serde_json::from_str::<Value>(&text) {
                        if value.get("type").and_then(Value::as_str) == Some("heartbeat") {
                            let _ = writer.send(Message::Text(r#"{"type":"heartbeat"}"#.into()));
                            continue;
                        }
                        if frame_tx.send(value).is_err() {
                            break;
                        }
                    }
                }
                Message::Ping(p) => {
                    let _ = writer.send(Message::Pong(p));
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        // Dropping frame_tx here ends the supervisor's `recv()` => disconnect.
    });

    Ok((tx, frame_rx))
}

/// Default read-idle timeout: with no frame (not even a heartbeat) for this long, the
/// link assumes the node is gone (e.g. paused / partitioned) and fails over. Overridable
/// via `NANO_LINK_IDLE_MS`. Adapted upward from the gateway's advertised heartbeat.
fn link_idle_default() -> Duration {
    Duration::from_millis(
        std::env::var("NANO_LINK_IDLE_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(4000),
    )
}

/// Handle one decoded server frame.
type FrameHook = Box<dyn Fn(&Value) + Send + Sync>;
/// Called right after a (re)connection, with the live writer — used to (re)send the
/// worker subscription so a failed-over worker keeps receiving jobs.
type ConnectHook = Box<dyn Fn(&mpsc::UnboundedSender<Message>) + Send + Sync>;
/// Called on every disconnect — used to fail in-flight waiters promptly.
type DisconnectHook = Box<dyn Fn() + Send + Sync>;

/// Hooks the [`SupervisedLink`] supervisor invokes across the connection lifecycle.
struct LinkHooks {
    on_frame: FrameHook,
    on_connect: ConnectHook,
    on_disconnect: DisconnectHook,
}

/// Shared, swappable state for a reconnecting falcon link.
struct LinkInner {
    /// Current live writer; `None` while disconnected/reconnecting.
    writer: Mutex<Option<mpsc::UnboundedSender<Message>>>,
    /// The failover directory (≥1 endpoint).
    endpoints: Vec<String>,
    /// Endpoint currently connected (for observability).
    current: Mutex<Option<String>>,
    /// Total successful (re)connections; `connects - 1` = failovers.
    connects: AtomicU64,
}

/// A falcon link that transparently fails over across a directory of nodes.
///
/// A background supervisor keeps exactly one socket alive: it picks an endpoint at random,
/// runs it until a disconnect or a read-idle timeout, then re-picks (avoiding the node
/// that just failed) and reconnects — re-running `on_connect`. Because any nanobpmn gateway
/// is a full proxy (create-forwarding + cluster-wide job aggregation), reconnecting to any
/// survivor restores whole-cluster access; rebalancing happens server-side.
struct SupervisedLink {
    inner: Arc<LinkInner>,
}

impl SupervisedLink {
    /// Start the supervisor and wait for the first connection before returning, so the
    /// caller can immediately send (e.g. the initial subscribe was already issued by
    /// `on_connect`).
    async fn start(endpoints: Vec<String>, hooks: LinkHooks) -> Result<SupervisedLink> {
        debug_assert!(!endpoints.is_empty());
        let inner = Arc::new(LinkInner {
            writer: Mutex::new(None),
            endpoints,
            current: Mutex::new(None),
            connects: AtomicU64::new(0),
        });
        let (ready_tx, ready_rx) = oneshot::channel::<Result<()>>();
        let sup = inner.clone();
        tokio::spawn(async move {
            supervise(sup, hooks, ready_tx).await;
        });
        // Surface the first connect result (so a bad address still fails fast → REST).
        match ready_rx.await {
            Ok(Ok(())) => Ok(SupervisedLink { inner }),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(CamundaError::Config("nano link supervisor exited".into())),
        }
    }

    /// Send a frame on the current socket. Returns an error while disconnected.
    fn send(&self, msg: Message) -> Result<()> {
        let guard = self.inner.writer.lock().unwrap();
        match guard.as_ref() {
            Some(tx) => tx
                .send(msg)
                .map_err(|_| CamundaError::Config("nano Falcon closed".into())),
            None => Err(CamundaError::Config(
                "nano Falcon reconnecting".into(),
            )),
        }
    }
}

/// Pick the next endpoint: random, but avoid `avoid` (the node that just failed) when the
/// directory has more than one entry. Cheap xorshift seeded from the clock — no rng dep.
fn pick_endpoint<'a>(endpoints: &'a [String], avoid: Option<&str>, seed: &mut u64) -> &'a str {
    if endpoints.len() == 1 {
        return &endpoints[0];
    }
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    let start = (*seed as usize) % endpoints.len();
    for i in 0..endpoints.len() {
        let cand = &endpoints[(start + i) % endpoints.len()];
        if Some(cand.as_str()) != avoid {
            return cand;
        }
    }
    &endpoints[start]
}

/// The reconnect loop. Runs until the process ends (the link lives for the client's life).
async fn supervise(inner: Arc<LinkInner>, hooks: LinkHooks, ready_tx: oneshot::Sender<Result<()>>) {
    let idle = link_idle_default();
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64 | 1)
        .unwrap_or(0x9E37_79B9_7F4A_7C15);
    let mut last_failed: Option<String> = None;
    let mut ready_tx = Some(ready_tx);

    loop {
        let url = pick_endpoint(&inner.endpoints, last_failed.as_deref(), &mut seed).to_string();
        match dial(&url).await {
            Ok((tx, mut frames)) => {
                *inner.writer.lock().unwrap() = Some(tx.clone());
                *inner.current.lock().unwrap() = Some(url.clone());
                let n = inner.connects.fetch_add(1, Ordering::SeqCst) + 1;
                if n == 1 {
                    if let Some(rt) = ready_tx.take() {
                        let _ = rt.send(Ok(()));
                    }
                } else {
                    tracing::info!(endpoint = %url, failovers = n - 1, "nano link reconnected");
                }
                (hooks.on_connect)(&tx);

                // Pump frames; a read-idle timeout means the node went away silently.
                loop {
                    match tokio::time::timeout(idle, frames.recv()).await {
                        Ok(Some(frame)) => (hooks.on_frame)(&frame),
                        Ok(None) => break, // socket closed
                        Err(_) => {
                            tracing::warn!(endpoint = %url, "nano link idle timeout; failing over");
                            break;
                        }
                    }
                }

                *inner.writer.lock().unwrap() = None;
                (hooks.on_disconnect)();
                last_failed = Some(url);
            }
            Err(e) => {
                // First-ever connection failed: report so the client falls back to REST.
                if let Some(rt) = ready_tx.take() {
                    let _ = rt.send(Err(e));
                    return;
                }
                last_failed = Some(url);
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Producer (create-over-stream)
// ----------------------------------------------------------------------------

/// Outcome of a create over the Falcon.
pub struct CreateOutcome {
    pub process_instance_key: String,
    pub process_completed: bool,
    /// Output variables, populated only for `awaitCompletion` creates.
    pub variables: Option<Value>,
}

struct PendingCreate {
    ack: oneshot::Sender<(u16, Value)>,
}

/// Shared state between [`NanoProducer`]'s public surface and its reader task. Kept in
/// its own `Arc` so the reader closure can capture it directly (no self-reference).
struct ProducerShared {
    credits: AtomicI64,
    credit_ready: Notify,
    pending: Mutex<HashMap<u64, PendingCreate>>,
    await_pending: Mutex<HashMap<u64, oneshot::Sender<(bool, Value)>>>,
}

impl ProducerShared {
    fn on_frame(&self, frame: &Value) {
        match frame.get("type").and_then(Value::as_str) {
            Some("welcome") => {
                if let Some(n) = frame.get("submissionCredits").and_then(Value::as_i64) {
                    self.credits.fetch_add(n, Ordering::SeqCst);
                    self.credit_ready.notify_waiters();
                }
            }
            Some("submissionCredits") => {
                if let Some(n) = frame.get("n").and_then(Value::as_i64) {
                    self.credits.fetch_add(n, Ordering::SeqCst);
                    self.credit_ready.notify_waiters();
                }
            }
            Some("commandResult") => {
                if let Some(corr) = frame.get("corr").and_then(Value::as_u64) {
                    let status = frame.get("status").and_then(Value::as_u64).unwrap_or(500) as u16;
                    let body = frame.get("body").cloned().unwrap_or(Value::Null);
                    if let Some(p) = self.pending.lock().unwrap().remove(&corr) {
                        let _ = p.ack.send((status, body));
                    }
                }
            }
            Some("instanceCompleted") => {
                if let Some(corr) = frame.get("corr").and_then(Value::as_u64) {
                    let completed = frame
                        .get("processCompleted")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    let variables = frame.get("variables").cloned().unwrap_or(Value::Null);
                    if let Some(tx) = self.await_pending.lock().unwrap().remove(&corr) {
                        let _ = tx.send((completed, variables));
                    }
                }
            }
            _ => {}
        }
    }

    /// On disconnect: reset the credit window (the next `Welcome` re-grants a fresh one)
    /// and fail every in-flight create promptly, so callers see an error and retry on the
    /// failed-over socket instead of blocking on a `CommandResult` that will never arrive.
    fn on_disconnect(&self) {
        self.credits.store(0, Ordering::SeqCst);
        self.pending.lock().unwrap().clear();
        self.await_pending.lock().unwrap().clear();
    }
}

/// A persistent, credit-metered create producer over a single falcon socket.
///
/// Cloneable handles share one socket. Creation is gated on the server-granted
/// submission-credit window: when credits are exhausted, [`NanoProducer::create`]
/// *waits* (no `503`, no client-side retry) until the gateway replenishes.
pub struct NanoProducer {
    link: SupervisedLink,
    next_corr: AtomicU64,
    shared: Arc<ProducerShared>,
    /// Maximum time to wait for a submission credit before giving up with a
    /// [`CamundaError::Backpressure`]. `None` waits indefinitely (the default,
    /// matching the historical behaviour). Set via `CAMUNDA_NANO_SUBMIT_TIMEOUT_MS`.
    submit_timeout: Option<Duration>,
}

impl NanoProducer {
    /// Connect a new producer over the failover directory `endpoints` (≥1). The supervisor
    /// picks one at random and fails over to a survivor on disconnect.
    ///
    /// `submit_timeout` bounds the per-create submission-credit wait: when the gateway
    /// withholds credits (admission backpressure) for longer than the budget,
    /// [`NanoProducer::create`] fails fast with a [`CamundaError::Backpressure`] instead of
    /// blocking. `None` waits indefinitely — the historical behaviour.
    pub async fn start(
        endpoints: Vec<String>,
        submit_timeout: Option<Duration>,
    ) -> Result<Arc<NanoProducer>> {
        let shared = Arc::new(ProducerShared {
            credits: AtomicI64::new(0),
            credit_ready: Notify::new(),
            pending: Mutex::new(HashMap::new()),
            await_pending: Mutex::new(HashMap::new()),
        });

        let frame_shared = shared.clone();
        let disc_shared = shared.clone();
        let hooks = LinkHooks {
            on_frame: Box::new(move |frame| frame_shared.on_frame(frame)),
            // The producer needs no re-handshake: each new connection's `Welcome` re-grants
            // a fresh submission-credit window.
            on_connect: Box::new(|_tx| {}),
            on_disconnect: Box::new(move || disc_shared.on_disconnect()),
        };
        let link = SupervisedLink::start(endpoints, hooks).await?;

        Ok(Arc::new(NanoProducer {
            link,
            next_corr: AtomicU64::new(1),
            shared,
            submit_timeout,
        }))
    }

    /// Take one submission credit, waiting for the gateway to replenish when exhausted.
    async fn acquire_credit(&self) {
        let credits = &self.shared.credits;
        let credit_ready = &self.shared.credit_ready;
        loop {
            // Register interest with the Notify BEFORE checking credits. `notify_waiters`
            // (fired when a reconnect's Welcome re-grants the credit window) only wakes
            // waiters that are already registered, so checking first and awaiting second
            // would lose the wakeup and deadlock the producer across a failover.
            let notified = credit_ready.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();

            let mut cur = credits.load(Ordering::SeqCst);
            while cur > 0 {
                match credits.compare_exchange_weak(
                    cur,
                    cur - 1,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => return,
                    Err(actual) => cur = actual,
                }
            }
            notified.await;
        }
    }

    /// [`acquire_credit`](Self::acquire_credit) bounded by `submit_timeout`. When the
    /// gateway withholds credits (admission backpressure) past the deadline, returns a
    /// [`CamundaError::Backpressure`] so the caller fails fast and can back off, rather
    /// than blocking indefinitely. With no timeout configured this waits forever.
    async fn acquire_credit_bounded(&self) -> Result<()> {
        match self.submit_timeout {
            Some(budget) => tokio::time::timeout(budget, self.acquire_credit())
                .await
                .map_err(|_| {
                    CamundaError::Backpressure(format!(
                        "no nano submission credit within {}ms (gateway is applying admission backpressure)",
                        budget.as_millis()
                    ))
                }),
            None => {
                self.acquire_credit().await;
                Ok(())
            }
        }
    }

    /// Create a process instance over the Falcon.
    pub async fn create(
        &self,
        process_definition_id: Option<&str>,
        process_definition_key: Option<&str>,
        variables: Option<Map<String, Value>>,
        await_completion: bool,
        fetch_variables: Option<Vec<String>>,
        request_timeout: Option<i64>,
    ) -> Result<CreateOutcome> {
        self.acquire_credit_bounded().await?;
        let corr = self.next_corr.fetch_add(1, Ordering::SeqCst);

        let (ack_tx, ack_rx) = oneshot::channel();
        self.shared
            .pending
            .lock()
            .unwrap()
            .insert(corr, PendingCreate { ack: ack_tx });

        let await_rx = if await_completion {
            let (tx, rx) = oneshot::channel();
            self.shared.await_pending.lock().unwrap().insert(corr, tx);
            Some(rx)
        } else {
            None
        };

        let mut frame = json!({
            "type": "createInstance",
            "corr": corr,
            "awaitCompletion": await_completion,
        });
        let obj = frame.as_object_mut().unwrap();
        if let Some(id) = process_definition_id {
            obj.insert("processDefinitionId".into(), json!(id));
        }
        if let Some(key) = process_definition_key {
            obj.insert("processDefinitionKey".into(), json!(key));
        }
        if let Some(vars) = variables {
            obj.insert("variables".into(), Value::Object(vars));
        }
        if let Some(fv) = fetch_variables {
            obj.insert("fetchVariables".into(), json!(fv));
        }
        if let Some(rt) = request_timeout {
            obj.insert("requestTimeout".into(), json!(rt));
        }

        self.link
            .send(Message::Text(frame.to_string()))
            .inspect_err(|_| {
                // Reconnecting/closed: release the slots we reserved for this corr.
                self.shared.pending.lock().unwrap().remove(&corr);
                self.shared.await_pending.lock().unwrap().remove(&corr);
            })?;

        let (status, body) = tokio::time::timeout(CREATE_ACK_TIMEOUT, ack_rx)
            .await
            .map_err(|_| CamundaError::Config("nano create timed out".into()))?
            .map_err(|_| CamundaError::Config("nano Falcon closed".into()))?;

        if status != 200 {
            self.shared.await_pending.lock().unwrap().remove(&corr);
            let detail = match &body {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            return Err(CamundaError::Api {
                status,
                body: Some(detail),
            });
        }

        let process_instance_key = body
            .get("processInstanceKey")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let mut process_completed = body
            .get("processCompleted")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let mut variables = None;

        if let Some(rx) = await_rx {
            // Bound the completion wait. `instanceCompleted` arrives only once a worker
            // finishes the job; if the owning node is paused/partitioned after the create
            // was acked, the frame may never arrive and no disconnect fires on this link.
            // Cap the wait at the caller's request timeout (a sane default otherwise) so a
            // stuck await-create errors and the caller can retry on a survivor instead of
            // blocking forever.
            let await_budget = request_timeout
                .filter(|ms| *ms > 0)
                .map(|ms| Duration::from_millis(ms as u64))
                .unwrap_or(CREATE_ACK_TIMEOUT);
            match tokio::time::timeout(await_budget, rx).await {
                Ok(Ok((completed, vars))) => {
                    process_completed = completed;
                    variables = Some(vars);
                }
                // Timed out, or the await channel was dropped on disconnect: surface an
                // error so the caller retries (the instance itself was created — the
                // gateway will still run it; this is an at-least-once create signal).
                _ => {
                    self.shared.await_pending.lock().unwrap().remove(&corr);
                    return Err(CamundaError::Config(
                        "nano await-completion timed out (node paused or partitioned)".into(),
                    ));
                }
            }
        }

        Ok(CreateOutcome {
            process_instance_key,
            process_completed,
            variables,
        })
    }
}

// ----------------------------------------------------------------------------
// Stream job worker
// ----------------------------------------------------------------------------

/// A push-based job worker over a single falcon socket.
///
/// Subscribes to one job type with an initial credit batch; the gateway pushes matching
/// jobs (each consuming a delivery credit). After acting on a job, the caller replenishes
/// one credit via [`NanoStreamWorker::replenish`], keeping in-flight work bounded by the
/// initial credit window (= `max_jobs_to_activate`).
pub struct NanoStreamWorker {
    link: SupervisedLink,
    jobs: tokio::sync::Mutex<mpsc::UnboundedReceiver<models::ActivatedJobResult>>,
    job_type: String,
}

impl NanoStreamWorker {
    /// Connect over the failover directory `endpoints` (≥1), subscribe to `job_type`, and
    /// start buffering pushed jobs. On failover the subscription is automatically re-sent to
    /// the survivor, so the worker keeps receiving jobs across a node pause/restart.
    pub async fn subscribe(
        endpoints: Vec<String>,
        job_type: &str,
        job_credits: i64,
        fetch_variable: Option<Vec<String>>,
        timeout_ms: Option<u64>,
        worker: Option<String>,
    ) -> Result<NanoStreamWorker> {
        let (jobs_tx, jobs_rx) = mpsc::unbounded_channel::<models::ActivatedJobResult>();

        let on_frame = move |frame: &Value| {
            if frame.get("type").and_then(Value::as_str) == Some("job") {
                if let Some(job) = frame.get("job").cloned() {
                    if let Ok(activated) = serde_json::from_value::<models::ActivatedJobResult>(job)
                    {
                        let _ = jobs_tx.send(activated);
                    }
                }
            }
        };

        // Build the subscribe frame once; (re)send it on every (re)connection.
        let mut sub = json!({
            "type": "subscribe",
            "jobType": job_type,
            "jobCredits": job_credits,
        });
        let obj = sub.as_object_mut().unwrap();
        if let Some(fv) = fetch_variable {
            obj.insert("fetchVariable".into(), json!(fv));
        }
        if let Some(t) = timeout_ms {
            obj.insert("timeout".into(), json!(t));
        }
        if let Some(w) = worker {
            obj.insert("worker".into(), json!(w));
        }
        let sub_text = sub.to_string();
        let on_connect = move |tx: &mpsc::UnboundedSender<Message>| {
            let _ = tx.send(Message::Text(sub_text.clone()));
        };

        let hooks = LinkHooks {
            on_frame: Box::new(on_frame),
            on_connect: Box::new(on_connect),
            on_disconnect: Box::new(|| {}),
        };
        let link = SupervisedLink::start(endpoints, hooks).await?;

        Ok(NanoStreamWorker {
            link,
            jobs: tokio::sync::Mutex::new(jobs_rx),
            job_type: job_type.to_string(),
        })
    }

    /// Await the next pushed job, or `None` if no job arrives within `wait`.
    pub async fn next_job(&self, wait: Duration) -> Option<models::ActivatedJobResult> {
        let mut rx = self.jobs.lock().await;
        tokio::time::timeout(wait, rx.recv())
            .await
            .unwrap_or_default()
    }

    /// Grant the gateway `n` more delivery credits for this worker's job type.
    pub fn replenish(&self, n: i64) {
        let frame = json!({"type": "jobCredits", "jobType": self.job_type, "n": n});
        let _ = self.link.send(Message::Text(frame.to_string()));
    }

    /// Complete a job (fire-and-forget) and replenish one credit.
    pub fn complete(&self, job_key: &str, variables: Option<Map<String, Value>>) {
        let mut frame = json!({"type": "completeJob", "corr": 0, "jobKey": job_key});
        if let Some(vars) = variables {
            frame
                .as_object_mut()
                .unwrap()
                .insert("variables".into(), Value::Object(vars));
        }
        let _ = self.link.send(Message::Text(frame.to_string()));
        self.replenish(1);
    }

    /// Fail a job (fire-and-forget) and replenish one credit.
    pub fn fail(&self, job_key: &str, retries: Option<i32>, error_message: Option<String>) {
        let mut frame = json!({"type": "failJob", "corr": 0, "jobKey": job_key});
        let obj = frame.as_object_mut().unwrap();
        if let Some(r) = retries {
            obj.insert("retries".into(), json!(r));
        }
        if let Some(m) = error_message {
            obj.insert("errorMessage".into(), json!(m));
        }
        let _ = self.link.send(Message::Text(frame.to_string()));
        self.replenish(1);
    }

    /// Throw a BPMN error from a job (fire-and-forget) and replenish one credit.
    pub fn throw_error(&self, job_key: &str, error_code: &str, error_message: Option<String>) {
        let mut frame = json!({
            "type": "throwError",
            "corr": 0,
            "jobKey": job_key,
            "errorCode": error_code,
        });
        if let Some(m) = error_message {
            frame
                .as_object_mut()
                .unwrap()
                .insert("errorMessage".into(), json!(m));
        }
        let _ = self.link.send(Message::Text(frame.to_string()));
        self.replenish(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_url_strips_v2_and_swaps_scheme() {
        assert_eq!(
            ws_url("http://localhost:8080/v2", "/falcon"),
            "ws://localhost:8080/falcon"
        );
        assert_eq!(
            ws_url("http://localhost:8080/v2/", "/falcon"),
            "ws://localhost:8080/falcon"
        );
        assert_eq!(
            ws_url("https://gw.example.com/v2", "/falcon"),
            "wss://gw.example.com/falcon"
        );
        // Path without leading slash is normalised.
        assert_eq!(
            ws_url("http://h:1/v2", "falcon"),
            "ws://h:1/falcon"
        );
        // No /v2 suffix: origin used as-is.
        assert_eq!(
            ws_url("http://h:1", "/falcon"),
            "ws://h:1/falcon"
        );
    }

    #[test]
    fn falcon_flag_defaults_on() {
        std::env::remove_var("CAMUNDA_NANO_FALCON");
        assert!(falcon_enabled());
        std::env::set_var("CAMUNDA_NANO_FALCON", "off");
        assert!(!falcon_enabled());
        std::env::set_var("CAMUNDA_NANO_FALCON", "1");
        assert!(falcon_enabled());
        std::env::remove_var("CAMUNDA_NANO_FALCON");
    }

    #[test]
    fn host_of_extracts_bare_host() {
        assert_eq!(
            host_of("http://127.0.0.1:8080/v2").as_deref(),
            Some("127.0.0.1")
        );
        assert_eq!(
            host_of("http://gw.example.com:8080").as_deref(),
            Some("gw.example.com")
        );
        assert_eq!(host_of("https://h").as_deref(), Some("h"));
    }

    #[test]
    fn endpoints_from_topology_builds_directory() {
        // A two-node cluster: node 0 reports itself as the self-placeholder 0.0.0.0,
        // which must be rewritten to the host we actually connected on (127.0.0.1).
        let body = serde_json::json!({
            "nano": { "falconPath": "/falcon" },
            "brokers": [
                { "nodeId": 0, "host": "0.0.0.0", "port": 8080 },
                { "nodeId": 1, "host": "127.0.0.1", "port": 8081 }
            ]
        });
        let eps = endpoints_from_topology("http://127.0.0.1:8080/v2", "/falcon", &body);
        assert!(eps.contains(&"ws://127.0.0.1:8080/falcon".to_string()));
        assert!(eps.contains(&"ws://127.0.0.1:8081/falcon".to_string()));
        // De-duplicated: the configured address coincides with node 0.
        assert_eq!(eps.len(), 2);
    }

    #[test]
    fn endpoints_from_topology_single_node_fallback() {
        // No brokers array → just the configured address.
        let body = serde_json::json!({ "nano": { "falconPath": "/falcon" } });
        let eps = endpoints_from_topology("http://localhost:8080/v2", "/falcon", &body);
        assert_eq!(eps, vec!["ws://localhost:8080/falcon".to_string()]);
    }

    #[test]
    fn pick_endpoint_avoids_failed_node() {
        let eps = vec![
            "ws://a/cs".to_string(),
            "ws://b/cs".to_string(),
            "ws://c/cs".to_string(),
        ];
        let mut seed = 0x1234_5678u64;
        // Over many picks avoiding "ws://b/cs", it must never be selected.
        for _ in 0..200 {
            let p = pick_endpoint(&eps, Some("ws://b/cs"), &mut seed);
            assert_ne!(p, "ws://b/cs");
        }
        // Single-element directory always returns its only entry, even if "avoided".
        let one = vec!["ws://solo/cs".to_string()];
        assert_eq!(
            pick_endpoint(&one, Some("ws://solo/cs"), &mut seed),
            "ws://solo/cs"
        );
    }
}
