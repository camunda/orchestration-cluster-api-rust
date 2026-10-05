//! Regression test for issue #59: a generated operation's typed error `entity` must
//! be the variant named by the HTTP status, not the first variant whose payload fits.
//!
//! The error enums are `#[serde(untagged)]` and almost every variant wraps
//! `ProblemDetail`, so decoding by shape alone returned `Status400` for a 404 or 409.
//! These tests drive the real generated operations against a one-shot local HTTP
//! server, so they exercise the generated call site and not only the decoder.
//! `scripts/test_hooks.py::EveryErrorEnumDecodesByStatusTest` guards the rest of the
//! 244 enums structurally.

use camunda_orchestration_sdk::apis::{
    audit_log_api::{self, SearchAuditLogsError, SearchAuditLogsParams},
    cluster_api::{self, GetClusterStatusError},
    configuration::Configuration,
    process_instance_api::{self, CancelProcessInstanceError, CancelProcessInstanceParams},
    Error, ResponseContent,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Serve exactly one HTTP response with `status` and `body`, returning the base path.
async fn serve_once(status: u16, body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        // None of the requests below send a body, so the headers are the whole request.
        let mut buf = Vec::new();
        let mut chunk = [0u8; 1024];
        while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
            let n = sock.read(&mut chunk).await.unwrap();
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
        }
        let response = format!(
            "HTTP/1.1 {status} X\r\nContent-Type: application/problem+json\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        sock.write_all(response.as_bytes()).await.unwrap();
        sock.shutdown().await.unwrap();
    });
    format!("http://{addr}")
}

fn config(base_path: String) -> Configuration {
    Configuration {
        base_path,
        ..Configuration::default()
    }
}

fn response_content<T>(err: Error<T>) -> ResponseContent<T> {
    match err {
        Error::ResponseError(content) => content,
        other => panic!("expected a ResponseError, got {other}"),
    }
}

const PROBLEM: &str =
    r#"{"type":"about:blank","title":"t","status":0,"detail":"d","instance":"/i"}"#;

async fn cancel(status: u16, body: &'static str) -> Option<CancelProcessInstanceError> {
    let cfg = config(serve_once(status, body).await);
    let params = CancelProcessInstanceParams {
        process_instance_key: "2251799813685249".into(),
        cancel_process_instance_request: None,
    };
    let err = process_instance_api::cancel_process_instance(&cfg, params)
        .await
        .expect_err("the server always answers with an error status");
    let content = response_content(err);
    assert_eq!(content.status.as_u16(), status);
    content.entity
}

/// Every status `cancel_process_instance` declares must come back as its own variant.
/// Before the fix, every one of these decoded as `Status400`.
#[tokio::test]
async fn cancel_process_instance_selects_the_variant_from_the_status() {
    for status in [400u16, 404, 409, 500, 503, 504] {
        let entity = cancel(status, PROBLEM).await;
        let got = match entity {
            Some(CancelProcessInstanceError::Status400(_)) => 400,
            Some(CancelProcessInstanceError::Status404(_)) => 404,
            Some(CancelProcessInstanceError::Status409(_)) => 409,
            Some(CancelProcessInstanceError::Status500(_)) => 500,
            Some(CancelProcessInstanceError::Status503(_)) => 503,
            Some(CancelProcessInstanceError::Status504(_)) => 504,
            other => panic!("status {status}: unexpected entity {other:?}"),
        };
        assert_eq!(got, status, "status {status} decoded as Status{got}");
    }
}

#[tokio::test]
async fn an_undeclared_status_decodes_as_unknown_value() {
    let entity = cancel(418, PROBLEM).await;
    assert!(
        matches!(entity, Some(CancelProcessInstanceError::UnknownValue(_))),
        "got {entity:?}"
    );
}

#[tokio::test]
async fn a_body_that_does_not_fit_the_status_variant_decodes_as_unknown_value() {
    let entity = cancel(409, r#"{"unexpected":true}"#).await;
    assert!(
        matches!(entity, Some(CancelProcessInstanceError::UnknownValue(_))),
        "got {entity:?}"
    );
}

#[tokio::test]
async fn a_non_json_body_has_no_entity() {
    assert!(cancel(409, "conflict").await.is_none());
}

/// `search_audit_logs` declares a 500 response with no schema, generated as `Status500()`.
/// Before the fix a ProblemDetail body under 500 decoded as `Status400`.
#[tokio::test]
async fn a_payload_less_variant_is_selected_by_status() {
    let cfg = config(serve_once(500, PROBLEM).await);
    let params = SearchAuditLogsParams {
        audit_log_search_query_request: None,
    };
    let err = audit_log_api::search_audit_logs(&cfg, params)
        .await
        .unwrap_err();
    let entity = response_content(err).entity;
    assert!(
        matches!(entity, Some(SearchAuditLogsError::Status500())),
        "got {entity:?}"
    );
}

/// A variant whose payload is not `ProblemDetail` still decodes into that payload.
#[tokio::test]
async fn a_non_problem_detail_payload_decodes_into_its_variant() {
    let cfg = config(serve_once(503, r#"{"status":"DOWN"}"#).await);
    let err = cluster_api::get_cluster_status(&cfg).await.unwrap_err();
    let entity = response_content(err).entity;
    assert!(
        matches!(entity, Some(GetClusterStatusError::Status503(_))),
        "got {entity:?}"
    );
}
