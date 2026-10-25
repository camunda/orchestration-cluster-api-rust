//! Regression coverage for the `truncateValues` query parameter on
//! `POST /variables/search` (PR #80 review): the curated
//! [`CamundaClient::search_variables`] wrapper must always send `truncateValues`
//! explicitly — `true` (the server-side default) when no option is given, and the
//! caller's explicit value otherwise. Reverting the field to `None`, or treating
//! `Some(false)` as "use the default", must fail these tests.
//!
//! These tests drive the real client against a one-shot local HTTP server and assert
//! the request line's target, so they capture the query string exactly as it goes on
//! the wire.

use camunda_orchestration_sdk::{models, CamundaClient, CamundaOptions};
use std::future::Future;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Serve exactly one HTTP response. Returns `(base_address, request_target)` where
/// `request_target` resolves to the request line's target (path + query) once a
/// request arrives — await it *after* issuing the call under test.
async fn serve_once(body: &'static str) -> (String, impl Future<Output = String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        let mut buf = Vec::new();
        let mut chunk = [0u8; 1024];
        while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
            let n = sock.read(&mut chunk).await.unwrap();
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
        }
        let request_line = String::from_utf8_lossy(&buf)
            .lines()
            .next()
            .unwrap_or_default()
            .to_string();
        let target = request_line
            .split_whitespace()
            .nth(1)
            .unwrap_or_default()
            .to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        sock.write_all(response.as_bytes()).await.unwrap();
        let _ = sock.shutdown().await;
        let _ = tx.send(target);
    });
    // The client normalizes the address by appending `/v2`; give it the host root so
    // the resulting base path is `http://<addr>/v2`.
    let base = format!("http://{addr}");
    (base, async move { rx.await.unwrap() })
}

/// Build a client pointed at the given local server, with no auth and no default
/// tenant — only the request target is under test.
fn client_at(base_address: &str) -> CamundaClient {
    let options = CamundaOptions::new().with("CAMUNDA_REST_ADDRESS", base_address);
    CamundaClient::new(options).expect("client builds against a local server")
}

const EMPTY_RESULT: &str = r#"{"page":{"totalItems":0,"hasMoreTotalItems":false,"startCursor":null,"endCursor":null},"items":[]}"#;

/// `search_variables` (no options) sends `truncateValues=true`, the server-side
/// default, so long values are truncated like the sibling SDKs.
#[tokio::test]
async fn search_variables_sends_truncate_values_true_by_default() {
    let (base, target) = serve_once(EMPTY_RESULT).await;
    let client = client_at(&base);
    let _ = client
        .search_variables(models::VariableSearchQuery::new())
        .await
        .expect("empty result deserializes");
    assert_eq!(target.await, "/v2/variables/search?truncateValues=true");
}

/// `search_variables_with_options(None)` likewise defaults to `truncateValues=true`.
#[tokio::test]
async fn search_variables_with_none_options_sends_truncate_values_true() {
    let (base, target) = serve_once(EMPTY_RESULT).await;
    let client = client_at(&base);
    let _ = client
        .search_variables_with_options(models::VariableSearchQuery::new(), None)
        .await
        .expect("empty result deserializes");
    assert_eq!(target.await, "/v2/variables/search?truncateValues=true");
}

/// An explicit `Some(false)` is sent verbatim as `truncateValues=false` so the caller
/// can read full, untruncated values — it must NOT collapse to the default.
#[tokio::test]
async fn search_variables_with_some_false_sends_truncate_values_false() {
    let (base, target) = serve_once(EMPTY_RESULT).await;
    let client = client_at(&base);
    let _ = client
        .search_variables_with_options(models::VariableSearchQuery::new(), Some(false))
        .await
        .expect("empty result deserializes");
    assert_eq!(target.await, "/v2/variables/search?truncateValues=false");
}

/// An explicit `Some(true)` is sent verbatim as `truncateValues=true`.
#[tokio::test]
async fn search_variables_with_some_true_sends_truncate_values_true() {
    let (base, target) = serve_once(EMPTY_RESULT).await;
    let client = client_at(&base);
    let _ = client
        .search_variables_with_options(models::VariableSearchQuery::new(), Some(true))
        .await
        .expect("empty result deserializes");
    assert_eq!(target.await, "/v2/variables/search?truncateValues=true");
}
