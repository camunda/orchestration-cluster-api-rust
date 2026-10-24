//! Regression test for issue #72: cluster-admin operations under `/cluster/v2/*`
//! declare a path-item `servers` override without the top-level `/v2` base path, so
//! `GET /cluster/v2/topology` must be requested as `/cluster/v2/topology` — not
//! `/v2/cluster/v2/topology`. These tests drive the real generated operations against
//! a one-shot local HTTP server and assert the request line's path.
//! `scripts/test_hooks.py::GeneratedClusterAdminBasePathTest` guards all 22 call
//! sites structurally; this file exercises one per affected API module end to end.

use camunda_orchestration_sdk::apis::{
    backup_api, cluster_api, configuration::Configuration, exporting_api, recovery_api,
};
use std::future::Future;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Serve exactly one HTTP response. Returns `(base_path, request_target)` where
/// `request_target` resolves to the request line's target once a request arrives —
/// await it *after* issuing the call under test.
async fn serve_once(status: u16, body: &'static str) -> (String, impl Future<Output = String>) {
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
            "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        sock.write_all(response.as_bytes()).await.unwrap();
        let _ = sock.shutdown().await;
        let _ = tx.send(target);
    });
    let base = format!("http://{addr}/v2");
    (base, async move { rx.await.unwrap() })
}

fn config(base_path: String) -> Configuration {
    Configuration {
        base_path,
        ..Configuration::default()
    }
}

/// `GET /cluster/v2/topology` — no `/v2` prefix, even though `base_path` ends in `/v2`.
#[tokio::test]
async fn cluster_topology_is_requested_from_the_server_root() {
    let (base, target) = serve_once(200, r#"{"brokers":[]}"#).await;
    let _ = cluster_api::get_cluster_topology(&config(base)).await;
    assert_eq!(target.await, "/cluster/v2/topology");
}

/// `GET /cluster/v2/status` (cluster_api).
#[tokio::test]
async fn cluster_status_is_requested_from_the_server_root() {
    let (base, target) = serve_once(200, r#"{"status":"HEALTHY"}"#).await;
    let _ = cluster_api::get_cluster_status(&config(base)).await;
    assert_eq!(target.await, "/cluster/v2/status");
}

/// `GET /cluster/v2/backups/runtime` (backup_api).
#[tokio::test]
async fn runtime_backups_are_requested_from_the_server_root() {
    let (base, target) = serve_once(200, r#"[]"#).await;
    let params = backup_api::ListRuntimeBackupsAsClusterAdminParams {
        physical_tenant_id: None,
        prefix: None,
    };
    let _ = backup_api::list_runtime_backups_as_cluster_admin(&config(base), params).await;
    assert_eq!(target.await, "/cluster/v2/backups/runtime");
}

/// `GET /cluster/v2/exporting` (exporting_api).
#[tokio::test]
async fn cluster_exporting_status_is_requested_from_the_server_root() {
    let (base, target) = serve_once(200, r#"{"status":"EXPORTING"}"#).await;
    let _ = exporting_api::get_cluster_exporting_status(&config(base)).await;
    assert_eq!(target.await, "/cluster/v2/exporting");
}

/// `POST /cluster/v2/restore` (recovery_api).
#[tokio::test]
async fn cluster_restore_is_requested_from_the_server_root() {
    let (base, target) = serve_once(202, r#"{"changeId":"c1","plannedChanges":[]}"#).await;
    let params = recovery_api::RestoreAsClusterAdminParams {
        cluster_restore_request: Default::default(),
        physical_tenant_id: None,
        dry_run: None,
    };
    let _ = recovery_api::restore_as_cluster_admin(&config(base), params).await;
    assert_eq!(target.await, "/cluster/v2/restore");
}

/// A regular operation on the same modules keeps the `/v2` base path.
#[tokio::test]
async fn non_admin_operations_keep_the_v2_base_path() {
    let (base, target) = serve_once(200, r#"{"brokers":[]}"#).await;
    let _ = cluster_api::get_topology(&config(base)).await;
    assert_eq!(target.await, "/v2/topology");
}

/// The helper itself: strips `/v2` only for the overridden paths, and tolerates a
/// base path without the suffix (e.g. `CAMUNDA_REST_ADDRESS_EXACT` gateway setups).
#[test]
fn operation_base_path_strips_v2_only_for_cluster_admin_paths() {
    let cfg = config("http://localhost:8080/v2".to_string());
    assert_eq!(
        cfg.operation_base_path("/cluster/v2/topology"),
        "http://localhost:8080"
    );
    assert_eq!(
        cfg.operation_base_path("/topology"),
        "http://localhost:8080/v2"
    );

    let exact = config("https://gateway.example/api/camunda".to_string());
    assert_eq!(
        exact.operation_base_path("/cluster/v2/status"),
        "https://gateway.example/api/camunda"
    );
}
