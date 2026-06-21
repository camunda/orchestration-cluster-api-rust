//! Nano-aware command-stream transport.
//!
//! [nanobpmn](https://github.com/jwulf/nano-bpm) is an API/behaviour superset of the
//! Camunda 8 Orchestration Cluster. In addition to the REST API this SDK targets, a
//! nanobpmn gateway exposes a persistent **command-stream** WebSocket that adds
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
//! `CAMUNDA_NANO_COMMAND_STREAM` (default on; set `off`/`false`/`0` to force pure REST).
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

/// Default command-stream path advertised by a nanobpmn gateway.
const DEFAULT_COMMAND_STREAM_PATH: &str = "/command-stream";

/// How long to wait for a create `CommandResult` before giving up.
const CREATE_ACK_TIMEOUT: Duration = Duration::from_secs(30);

/// Detected nanobpmn capabilities, derived from `GET /v2/topology`.
#[derive(Debug, Clone)]
pub struct NanoCaps {
    /// WebSocket command-stream path (e.g. `/command-stream`).
    pub command_stream_path: String,
}

/// Returns `true` unless `CAMUNDA_NANO_COMMAND_STREAM` is explicitly disabled.
pub fn command_stream_enabled() -> bool {
    match std::env::var("CAMUNDA_NANO_COMMAND_STREAM") {
        Ok(v) => !matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "0" | "off" | "false" | "no"
        ),
        Err(_) => true,
    }
}

/// Probe a gateway's `/v2/topology` for the `nano` advertisement.
///
/// Returns `None` for stock Camunda (no `nano` field), when the command stream is
/// disabled by config, when the gateway is not reachable over plaintext `ws://`, or on
/// any error — in every case the caller falls back to REST, so detection never fails a
/// request.
pub async fn detect(rest_address: &str, http: &reqwest::Client) -> Option<NanoCaps> {
    if !command_stream_enabled() {
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
    let command_stream_path = nano
        .get("commandStreamPath")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_COMMAND_STREAM_PATH)
        .to_string();
    Some(NanoCaps {
        command_stream_path,
    })
}

/// Derive the command-stream WebSocket URL from the REST address.
///
/// `rest_address` is normalised to `<scheme>://host:port/v2`; the command stream lives
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

/// Open a command-stream WebSocket and split it into a writer task (fed by the returned
/// channel) and a reader stream handed to `on_frame` for each decoded server frame.
///
/// The writer task drains `rx` until the channel closes (i.e. the last sender drops),
/// then the socket is closed. `on_frame` receives every text frame as parsed JSON.
async fn connect<F>(url: &str, on_frame: F) -> Result<mpsc::UnboundedSender<Message>>
where
    F: Fn(Value) + Send + 'static,
{
    let (ws, _resp) = tokio_tungstenite::connect_async(url)
        .await
        .map_err(|e| CamundaError::Config(format!("nano command-stream connect failed: {e}")))?;
    let (mut sink, mut stream) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Writer task: forward queued frames to the socket.
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
        let _ = sink.close().await;
    });

    // Reader task: decode text frames and dispatch.
    let writer = tx.clone();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(value) = serde_json::from_str::<Value>(&text) {
                        // Answer heartbeats so the gateway doesn't reap an idle socket.
                        if value.get("type").and_then(Value::as_str) == Some("heartbeat") {
                            let _ = writer.send(Message::Text(r#"{"type":"heartbeat"}"#.into()));
                            continue;
                        }
                        on_frame(value);
                    }
                }
                Message::Ping(p) => {
                    let _ = writer.send(Message::Pong(p));
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    Ok(tx)
}

// ----------------------------------------------------------------------------
// Producer (create-over-stream)
// ----------------------------------------------------------------------------

/// Outcome of a create over the command stream.
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
    fn on_frame(&self, frame: Value) {
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
}

/// A persistent, credit-metered create producer over a single command-stream socket.
///
/// Cloneable handles share one socket. Creation is gated on the server-granted
/// submission-credit window: when credits are exhausted, [`NanoProducer::create`]
/// *waits* (no `503`, no client-side retry) until the gateway replenishes.
pub struct NanoProducer {
    tx: mpsc::UnboundedSender<Message>,
    next_corr: AtomicU64,
    shared: Arc<ProducerShared>,
}

impl NanoProducer {
    /// Connect a new producer socket to `ws_url`.
    pub async fn connect(ws_url: &str) -> Result<Arc<NanoProducer>> {
        let shared = Arc::new(ProducerShared {
            credits: AtomicI64::new(0),
            credit_ready: Notify::new(),
            pending: Mutex::new(HashMap::new()),
            await_pending: Mutex::new(HashMap::new()),
        });

        let reader_shared = shared.clone();
        let tx = connect(ws_url, move |frame| reader_shared.on_frame(frame)).await?;

        Ok(Arc::new(NanoProducer {
            tx,
            next_corr: AtomicU64::new(1),
            shared,
        }))
    }

    /// Take one submission credit, waiting for the gateway to replenish when exhausted.
    async fn acquire_credit(&self) {
        let credits = &self.shared.credits;
        let credit_ready = &self.shared.credit_ready;
        loop {
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
            credit_ready.notified().await;
        }
    }

    /// Create a process instance over the command stream.
    pub async fn create(
        &self,
        process_definition_id: Option<&str>,
        process_definition_key: Option<&str>,
        variables: Option<Map<String, Value>>,
        await_completion: bool,
        fetch_variables: Option<Vec<String>>,
        request_timeout: Option<i64>,
    ) -> Result<CreateOutcome> {
        self.acquire_credit().await;
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

        self.tx
            .send(Message::Text(frame.to_string()))
            .map_err(|_| CamundaError::Config("nano command stream closed".into()))?;

        let (status, body) = tokio::time::timeout(CREATE_ACK_TIMEOUT, ack_rx)
            .await
            .map_err(|_| CamundaError::Config("nano create timed out".into()))?
            .map_err(|_| CamundaError::Config("nano command stream closed".into()))?;

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
            if let Ok((completed, vars)) = rx.await {
                process_completed = completed;
                variables = Some(vars);
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

/// A push-based job worker over a single command-stream socket.
///
/// Subscribes to one job type with an initial credit batch; the gateway pushes matching
/// jobs (each consuming a delivery credit). After acting on a job, the caller replenishes
/// one credit via [`NanoStreamWorker::replenish`], keeping in-flight work bounded by the
/// initial credit window (= `max_jobs_to_activate`).
pub struct NanoStreamWorker {
    tx: mpsc::UnboundedSender<Message>,
    jobs: tokio::sync::Mutex<mpsc::UnboundedReceiver<models::ActivatedJobResult>>,
    job_type: String,
}

impl NanoStreamWorker {
    /// Connect, subscribe to `job_type`, and start buffering pushed jobs.
    pub async fn subscribe(
        ws_url: &str,
        job_type: &str,
        job_credits: i64,
        fetch_variable: Option<Vec<String>>,
        timeout_ms: Option<u64>,
        worker: Option<String>,
    ) -> Result<NanoStreamWorker> {
        let (jobs_tx, jobs_rx) = mpsc::unbounded_channel::<models::ActivatedJobResult>();

        let on_frame = move |frame: Value| {
            if frame.get("type").and_then(Value::as_str) == Some("job") {
                if let Some(job) = frame.get("job").cloned() {
                    if let Ok(activated) = serde_json::from_value::<models::ActivatedJobResult>(job)
                    {
                        let _ = jobs_tx.send(activated);
                    }
                }
            }
        };

        let tx = connect(ws_url, on_frame).await?;

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
        tx.send(Message::Text(sub.to_string()))
            .map_err(|_| CamundaError::Config("nano command stream closed".into()))?;

        Ok(NanoStreamWorker {
            tx,
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
        let _ = self.tx.send(Message::Text(frame.to_string()));
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
        let _ = self.tx.send(Message::Text(frame.to_string()));
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
        let _ = self.tx.send(Message::Text(frame.to_string()));
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
        let _ = self.tx.send(Message::Text(frame.to_string()));
        self.replenish(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_url_strips_v2_and_swaps_scheme() {
        assert_eq!(
            ws_url("http://localhost:8080/v2", "/command-stream"),
            "ws://localhost:8080/command-stream"
        );
        assert_eq!(
            ws_url("http://localhost:8080/v2/", "/command-stream"),
            "ws://localhost:8080/command-stream"
        );
        assert_eq!(
            ws_url("https://gw.example.com/v2", "/command-stream"),
            "wss://gw.example.com/command-stream"
        );
        // Path without leading slash is normalised.
        assert_eq!(
            ws_url("http://h:1/v2", "command-stream"),
            "ws://h:1/command-stream"
        );
        // No /v2 suffix: origin used as-is.
        assert_eq!(
            ws_url("http://h:1", "/command-stream"),
            "ws://h:1/command-stream"
        );
    }

    #[test]
    fn command_stream_flag_defaults_on() {
        std::env::remove_var("CAMUNDA_NANO_COMMAND_STREAM");
        assert!(command_stream_enabled());
        std::env::set_var("CAMUNDA_NANO_COMMAND_STREAM", "off");
        assert!(!command_stream_enabled());
        std::env::set_var("CAMUNDA_NANO_COMMAND_STREAM", "1");
        assert!(command_stream_enabled());
        std::env::remove_var("CAMUNDA_NANO_COMMAND_STREAM");
    }
}
