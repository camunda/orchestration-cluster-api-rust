//! Sentinel-secret regression test, required of every SDK by the shared policy
//! ("Secrets must not leak"): configure recognisable sentinel credentials, drive the auth
//! success and failure paths, and assert the sentinels never appear in logs, errors or
//! `Debug` output.
//!
//! The assertions are substring scans over a single `SECRETS` table, so they catch the
//! defect class (a credential reaching any rendered surface) rather than today's call
//! sites. When a new place a credential can flow is added, add the surface to a scenario
//! below; when a new credential variable is added, add it to `SECRETS` and every scan
//! covers it.
//!
//! A failed assertion never prints the text it scanned: that text contains the credential
//! that leaked, and CI logs are retained. The panic names the variable and the surface
//! instead; reproduce locally to see the output.
//!
//! Out of scope on purpose: the Orchestration API's own error body. `CamundaError::Api`
//! documents that it carries the server's response body (see the job-variable trust model
//! in the README), so the REST stub below never echoes request data.

use camunda_orchestration_sdk::{CamundaClient, CamundaConfig, CamundaError, CamundaOptions};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const CLIENT_SECRET: &str = "SENTINEL-CLIENT-SECRET-d41d8c";
const BASIC_PASSWORD: &str = "SENTINEL-BASIC-PASSWORD-9e107d";
const MTLS_KEY: &str = "SENTINEL-MTLS-KEY-e4d909c2";
const MTLS_CERT: &str = "SENTINEL-MTLS-CERT-0cc175b9";
const MTLS_CA: &str = "SENTINEL-MTLS-CA-8277e091";
const MTLS_PASSPHRASE: &str = "SENTINEL-MTLS-PASSPHRASE-92eb5f";
const URL_PASSWORD: &str = "SENTINEL-URL-PASSWORD-1679091c";
/// The bearer token the stub identity provider issues. It is a credential too: it must not
/// be logged or put in an error, and it is the one value legitimately written to the cache
/// file.
const BEARER_TOKEN: &str = "SENTINEL-BEARER-TOKEN-5f4dcc3b";
/// Planted in the identity provider's error body, next to the reflected request.
const IDP_BODY: &str = "SENTINEL-IDP-BODY-c4ca4238";

/// Every credential the SDK is configured with, keyed by the variable that carries it.
const SECRETS: &[(&str, &str)] = &[
    ("CAMUNDA_CLIENT_SECRET", CLIENT_SECRET),
    ("CAMUNDA_BASIC_AUTH_PASSWORD", BASIC_PASSWORD),
    ("CAMUNDA_MTLS_KEY", MTLS_KEY),
    ("CAMUNDA_MTLS_CERT", MTLS_CERT),
    ("CAMUNDA_MTLS_CA", MTLS_CA),
    ("CAMUNDA_MTLS_KEY_PASSPHRASE", MTLS_PASSPHRASE),
];

/// A sentinel to scan for, and the human-readable name of what it stands for. The label is
/// the only part that ever reaches a panic message.
struct Needle<'a> {
    label: &'static str,
    value: &'a str,
}

/// Assert none of `needles` occurs in `rendered`. Never prints `rendered` or a sentinel.
fn assert_clean(surface: &str, rendered: &str, needles: &[Needle]) {
    for needle in needles {
        assert!(
            !rendered.contains(needle.value),
            "{} leaked into {surface} (output withheld: it contains the credential; \
             reproduce locally with `cargo test --test secret_redaction`)",
            needle.label
        );
    }
}

fn sentinels() -> Vec<Needle<'static>> {
    let mut v: Vec<Needle<'static>> = SECRETS
        .iter()
        .map(|&(label, value)| Needle { label, value })
        .collect();
    v.push(Needle {
        label: "the bearer token",
        value: BEARER_TOKEN,
    });
    v.push(Needle {
        label: "the identity-provider response body",
        value: IDP_BODY,
    });
    v.push(Needle {
        label: "the userinfo password of a configured URL",
        value: URL_PASSWORD,
    });
    v
}

/// Scan both renderings of an error, since callers use either.
fn assert_error_clean(surface: &str, err: &CamundaError, needles: &[Needle]) {
    assert_clean(&format!("{surface} (Display)"), &format!("{err}"), needles);
    assert_clean(&format!("{surface} (Debug)"), &format!("{err:?}"), needles);
}

// --- log capture ----------------------------------------------------------------------

/// A shared in-memory sink for a `tracing` subscriber.
#[derive(Clone, Default)]
struct LogCapture(Arc<Mutex<Vec<u8>>>);

impl io::Write for LogCapture {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl LogCapture {
    /// Capture everything, from every crate, at TRACE on the current thread. The tests run
    /// on a current-thread runtime, so every task they spawn is on this thread too.
    fn install(&self) -> tracing::subscriber::DefaultGuard {
        let sink = self.clone();
        tracing::subscriber::set_default(
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::TRACE)
                .with_ansi(false)
                .with_writer(move || sink.clone())
                .finish(),
        )
    }

    fn contents(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
    }
}

// --- in-process identity provider and API ---------------------------------------------

/// One request the stub served.
struct Seen {
    path: String,
    authorization: Option<String>,
    body: String,
}

struct Stub {
    base: String,
    seen: Arc<Mutex<Vec<Seen>>>,
}

impl Stub {
    fn requests_to(&self, suffix: &str) -> Vec<(Option<String>, String)> {
        self.seen
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.path.ends_with(suffix))
            .map(|s| (s.authorization.clone(), s.body.clone()))
            .collect()
    }
}

/// A dependency-free HTTP stub (the pattern `job_worker.rs` and the other integration
/// tests use) serving the token endpoint and `GET .../topology`.
///
/// `token_ok` selects a 200 with a token or a 401 whose body reflects the request —
/// including the posted `client_secret` — next to a planted sentinel, the way a verbose or
/// misrouted identity provider would. `rest_ok` selects a 200 topology or a bare 401.
async fn start_stub(token_ok: bool, rest_ok: bool) -> Stub {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let seen: Arc<Mutex<Vec<Seen>>> = Arc::default();
    let log = seen.clone();
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let log = log.clone();
            tokio::spawn(async move {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 2048];
                let header_end = loop {
                    match sock.read(&mut chunk).await {
                        Ok(0) | Err(_) => return,
                        Ok(n) => {
                            buf.extend_from_slice(&chunk[..n]);
                            if let Some(p) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                                break p + 4;
                            }
                        }
                    }
                };
                let head = String::from_utf8_lossy(&buf[..header_end]).into_owned();
                let header = |name: &str| {
                    head.lines().find_map(|l| {
                        let (k, v) = l.split_once(':')?;
                        k.eq_ignore_ascii_case(name).then(|| v.trim().to_string())
                    })
                };
                // Drain the body: closing with it unread can reset the connection and turn
                // the scenario into a transport failure.
                let len = header("content-length")
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(0);
                while buf.len() < header_end + len {
                    match sock.read(&mut chunk).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    }
                }
                let body = String::from_utf8_lossy(&buf[header_end..]).into_owned();
                let path = head
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or_default()
                    .to_string();
                log.lock().unwrap().push(Seen {
                    path: path.clone(),
                    authorization: header("authorization"),
                    body: body.clone(),
                });

                let (status, payload) = if path.ends_with("/oauth/token") {
                    if token_ok {
                        (
                            "200 OK",
                            format!(
                                r#"{{"access_token":"{BEARER_TOKEN}","expires_in":3600,"token_type":"Bearer"}}"#
                            ),
                        )
                    } else {
                        (
                            "401 Unauthorized",
                            format!(
                                r#"{{"error":"invalid_client","request":"{body}","detail":"{IDP_BODY}"}}"#
                            ),
                        )
                    }
                } else if path.ends_with("/topology") {
                    if rest_ok {
                        (
                            "200 OK",
                            r#"{"brokers":[],"clusterId":null,"clusterSize":1,"partitionsCount":1,"replicationFactor":1,"gatewayVersion":"8.11.0","lastCompletedChangeId":"0"}"#.to_string(),
                        )
                    } else {
                        (
                            "401 Unauthorized",
                            r#"{"title":"Unauthorized","status":401}"#.to_string(),
                        )
                    }
                } else {
                    ("404 Not Found", "{}".to_string())
                };
                let resp = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                    payload.len()
                );
                let _ = sock.write_all(resp.as_bytes()).await;
                let _ = sock.flush().await;
            });
        }
    });
    Stub {
        base: format!("http://{addr}"),
        seen,
    }
}

/// A scratch cache directory inside the system temp dir, removed on drop. Not a `tempfile`
/// dependency: the crate has none, and a redaction test should not add one.
struct ScratchDir(PathBuf);

impl ScratchDir {
    fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "camunda-secret-redaction-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        ScratchDir(dir)
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn options(pairs: &[(&str, &str)]) -> CamundaOptions {
    pairs
        .iter()
        .fold(CamundaOptions::new(), |o, (k, v)| o.with(*k, *v))
}

/// Surfaces of a built client that a caller can format: its resolved config and the
/// nested TLS config. (`CamundaClient` itself does not implement `Debug`.)
fn assert_client_surfaces_clean(client: &CamundaClient, needles: &[Needle]) {
    let config = client.config();
    assert_clean("CamundaConfig Debug", &format!("{config:?}"), needles);
    assert_clean(
        "CamundaConfig pretty Debug",
        &format!("{config:#?}"),
        needles,
    );
    assert_clean("TlsConfig Debug", &format!("{:?}", config.tls), needles);
}

// --- scenarios ------------------------------------------------------------------------

/// The scans over the captured logs only mean something if the capture works. Emit an event
/// the way the SDK does and require it to arrive, at TRACE and from a spawned task.
#[tokio::test]
async fn log_capture_sees_sdk_style_events() {
    let logs = LogCapture::default();
    let _guard = logs.install();
    tracing::trace!(target: "camunda_orchestration_sdk", "capture-self-check-trace");
    tokio::spawn(async { tracing::warn!(error = %"x", "capture-self-check-spawned") })
        .await
        .unwrap();
    let captured = logs.contents();
    assert!(captured.contains("capture-self-check-trace"));
    assert!(captured.contains("capture-self-check-spawned"));
}

/// OAuth success: the token is fetched, cached to disk and applied. The secret is posted to
/// the identity provider (and only there); it must not reach the logs, an error, `Debug`
/// output or the cache file.
#[tokio::test]
async fn oauth_success_path_does_not_leak() {
    let logs = LogCapture::default();
    let _guard = logs.install();
    let stub = start_stub(true, true).await;
    let cache = ScratchDir::new("oauth-success");

    let client = CamundaClient::new(options(&[
        ("CAMUNDA_REST_ADDRESS", &stub.base),
        ("CAMUNDA_AUTH_STRATEGY", "OAUTH"),
        ("CAMUNDA_OAUTH_URL", &format!("{}/oauth/token", stub.base)),
        ("CAMUNDA_CLIENT_ID", "sentinel-client"),
        ("CAMUNDA_CLIENT_SECRET", CLIENT_SECRET),
        ("CAMUNDA_OAUTH_CACHE_DIR", cache.0.to_str().unwrap()),
    ]))
    .expect("client builds");

    client.topology().await.expect("OAuth success path");

    // Not vacuous: the credential really was sent, and the token really was applied.
    let token_requests = stub.requests_to("/oauth/token");
    assert_eq!(token_requests.len(), 1, "exactly one token request");
    assert!(
        token_requests[0].1.contains(CLIENT_SECRET),
        "the client secret must be posted to the token endpoint"
    );
    assert_eq!(
        stub.requests_to("/topology")[0].0.as_deref(),
        Some(format!("Bearer {BEARER_TOKEN}").as_str()),
        "the issued token must be applied as a bearer credential"
    );

    let needles = sentinels();
    assert_clean("the captured logs", &logs.contents(), &needles);
    assert_client_surfaces_clean(&client, &needles);

    // The cache file holds the bearer token by design, and nothing else secret.
    let cached: Vec<_> = std::fs::read_dir(&cache.0)
        .expect("cache dir was created")
        .map(|e| std::fs::read_to_string(e.unwrap().path()).unwrap())
        .collect();
    assert!(!cached.is_empty(), "a token cache file was written");
    for contents in &cached {
        assert_clean(
            "the on-disk token cache",
            contents,
            &[
                Needle {
                    label: "CAMUNDA_CLIENT_SECRET",
                    value: CLIENT_SECRET,
                },
                Needle {
                    label: "the identity-provider response body",
                    value: IDP_BODY,
                },
            ],
        );
    }
}

/// OAuth failure: the identity provider rejects the client with a body that reflects the
/// request — `client_secret` included — and carries a planted sentinel. None of it may
/// reach the returned error or the logs.
#[tokio::test]
async fn oauth_failure_path_does_not_leak() {
    let logs = LogCapture::default();
    let _guard = logs.install();
    let stub = start_stub(false, true).await;

    let client = CamundaClient::new(options(&[
        ("CAMUNDA_REST_ADDRESS", &stub.base),
        ("CAMUNDA_AUTH_STRATEGY", "OAUTH"),
        ("CAMUNDA_OAUTH_URL", &format!("{}/oauth/token", stub.base)),
        ("CAMUNDA_CLIENT_ID", "sentinel-client"),
        ("CAMUNDA_CLIENT_SECRET", CLIENT_SECRET),
        // One attempt: the point is the failure's rendering, not the retry cadence.
        ("CAMUNDA_SDK_HTTP_RETRY_MAX_ATTEMPTS", "1"),
    ]))
    .expect("client builds");

    let err = client
        .topology()
        .await
        .expect_err("a rejected client must fail");
    assert!(
        matches!(err, CamundaError::Auth(_)),
        "expected an auth error, got variant {:?}",
        std::mem::discriminant(&err)
    );

    // Not vacuous: the stub really received the secret it was about to reflect.
    assert!(
        stub.requests_to("/oauth/token")[0]
            .1
            .contains(CLIENT_SECRET),
        "the client secret must be posted to the token endpoint"
    );

    let needles = sentinels();
    assert_error_clean("the OAuth failure error", &err, &needles);
    assert_clean("the captured logs", &logs.contents(), &needles);
    assert_client_surfaces_clean(&client, &needles);
}

/// Basic auth, success and failure. The password travels base64-encoded in a header, so the
/// encoded form is scanned for as well as the plain one.
#[tokio::test]
async fn basic_auth_paths_do_not_leak() {
    let logs = LogCapture::default();
    let _guard = logs.install();
    let encoded = base64(&format!("sentinel-user:{BASIC_PASSWORD}"));
    let mut needles: Vec<Needle> = sentinels();
    needles.push(Needle {
        label: "the base64-encoded basic credential",
        value: encoded.as_str(),
    });

    for rest_ok in [true, false] {
        let stub = start_stub(true, rest_ok).await;
        let client = CamundaClient::new(options(&[
            ("CAMUNDA_REST_ADDRESS", &stub.base),
            ("CAMUNDA_AUTH_STRATEGY", "BASIC"),
            ("CAMUNDA_BASIC_AUTH_USERNAME", "sentinel-user"),
            ("CAMUNDA_BASIC_AUTH_PASSWORD", BASIC_PASSWORD),
            ("CAMUNDA_SDK_HTTP_RETRY_MAX_ATTEMPTS", "1"),
        ]))
        .expect("client builds");

        let outcome = client.topology().await;
        assert_eq!(outcome.is_ok(), rest_ok, "scenario outcome");
        if let Err(err) = &outcome {
            assert_error_clean("the basic-auth failure error", err, &needles);
        }

        // Not vacuous: the credential was sent as a Basic header.
        let sent = stub.requests_to("/topology")[0]
            .0
            .clone()
            .unwrap_or_default();
        assert!(
            sent == format!("Basic {encoded}"),
            "the Basic header must carry exactly the configured credential \
             (headers withheld: they hold the credential)"
        );

        assert_client_surfaces_clean(&client, &needles);
    }
    assert_clean("the captured logs", &logs.contents(), &needles);
}

/// Misconfigured mutual TLS fails client construction. The error names the variable, never
/// the value; the inline values here are not valid PEM, so the backend's parse error is the
/// route a value would take into the message. Backend-independent: the CA value is
/// rejected eagerly by `native-tls` and deferred by `rustls`, so either outcome is scanned.
#[tokio::test]
async fn mtls_misconfiguration_does_not_leak() {
    let logs = LogCapture::default();
    let _guard = logs.install();
    let needles = sentinels();

    // Each of these must fail construction with a configuration error on every backend.
    let must_fail: &[(&str, &[(&str, &str)])] = &[
        // Certificate and key without a passphrase: the backend parses them.
        (
            "malformed identity",
            &[
                ("CAMUNDA_MTLS_CERT", MTLS_CERT),
                ("CAMUNDA_MTLS_KEY", MTLS_KEY),
            ],
        ),
        // With a passphrase: rejected as unsupported before any parse.
        (
            "identity with a passphrase",
            &[
                ("CAMUNDA_MTLS_CERT", MTLS_CERT),
                ("CAMUNDA_MTLS_KEY", MTLS_KEY),
                ("CAMUNDA_MTLS_KEY_PASSPHRASE", MTLS_PASSPHRASE),
            ],
        ),
        // A half identity.
        ("half identity", &[("CAMUNDA_MTLS_KEY", MTLS_KEY)]),
    ];
    for (name, pairs) in must_fail {
        match CamundaClient::new(options(pairs)) {
            Err(err @ CamundaError::Config(_)) => {
                assert_error_clean(&format!("the {name} error"), &err, &needles)
            }
            Err(_) => panic!("the {name} scenario must fail with a configuration error"),
            Ok(_) => panic!("the {name} scenario must fail client construction"),
        }
    }

    // A malformed CA is the one backend-dependent case: `native-tls` rejects it eagerly,
    // `rustls` defers it (see the mTLS tests in `tls.rs`). Either outcome is scanned.
    match CamundaClient::new(options(&[("CAMUNDA_MTLS_CA", MTLS_CA)])) {
        Err(err) => assert_error_clean("the malformed CA error", &err, &needles),
        Ok(client) => assert_client_surfaces_clean(&client, &needles),
    }
    assert_clean("the captured logs", &logs.contents(), &needles);
}

/// A password embedded in a configured URL (`https://user:password@host`) is a credential
/// too, and a transport failure renders the URL it was talking to.
#[tokio::test]
async fn url_userinfo_does_not_leak_through_transport_errors() {
    let logs = LogCapture::default();
    let _guard = logs.install();

    // A port nothing is listening on: connection refused.
    let port = {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        l.local_addr().unwrap().port()
    };
    let address = format!("http://sentinel-user:{URL_PASSWORD}@127.0.0.1:{port}");
    let client = CamundaClient::new(options(&[
        ("CAMUNDA_REST_ADDRESS", &address),
        ("CAMUNDA_AUTH_STRATEGY", "NONE"),
        ("CAMUNDA_SDK_HTTP_RETRY_MAX_ATTEMPTS", "1"),
    ]))
    .expect("client builds");

    let err = client
        .topology()
        .await
        .expect_err("nothing is listening, the call must fail");

    let needles = sentinels();
    assert_error_clean("the transport error", &err, &needles);
    assert_clean("the captured logs", &logs.contents(), &needles);
    assert_client_surfaces_clean(&client, &needles);
}

/// `CamundaConfig` and `TlsConfig` are public and formattable without any client: every
/// credential variable at once, resolved and rendered.
#[test]
fn resolved_config_debug_does_not_leak() {
    let mut overrides: HashMap<String, String> = SECRETS
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect();
    overrides.insert("CAMUNDA_AUTH_STRATEGY".into(), "BASIC".into());
    overrides.insert("CAMUNDA_BASIC_AUTH_USERNAME".into(), "sentinel-user".into());
    overrides.insert(
        "CAMUNDA_REST_ADDRESS".into(),
        format!("https://sentinel-user:{URL_PASSWORD}@cluster.example"),
    );
    overrides.insert(
        "CAMUNDA_OAUTH_URL".into(),
        format!("https://sentinel-user:{URL_PASSWORD}@idp.example/oauth/token"),
    );
    let config = CamundaConfig::from_env_with_overrides(&overrides).expect("config resolves");

    let needles = sentinels();
    for (surface, rendered) in [
        ("CamundaConfig Debug", format!("{config:?}")),
        ("CamundaConfig pretty Debug", format!("{config:#?}")),
        ("TlsConfig Debug", format!("{:?}", config.tls)),
        ("TlsConfig pretty Debug", format!("{:#?}", config.tls)),
    ] {
        assert_clean(surface, &rendered, &needles);
    }
}

/// URL parsers drop tab, carriage return and line feed wherever they occur, so
/// `https:\n//user:password@host` is the URL `https://user:password@host` — and the config
/// stores the value verbatim. A separator split by one of those characters must not hide the
/// userinfo from redaction.
#[test]
fn urls_with_control_characters_in_the_scheme_separator_do_not_leak() {
    let needles = sentinels();
    for split in ["https:\n//", "https:\t//", "https:/\r/", "https\n://"] {
        for var in ["CAMUNDA_REST_ADDRESS", "CAMUNDA_OAUTH_URL"] {
            let overrides: HashMap<String, String> = [(
                var.to_string(),
                format!("{split}sentinel-user:{URL_PASSWORD}@cluster.example/v2"),
            )]
            .into();
            let config =
                CamundaConfig::from_env_with_overrides(&overrides).expect("config resolves");
            assert_clean(
                &format!("CamundaConfig Debug of a {var} with a split scheme separator"),
                &format!("{config:?}"),
                &needles,
            );
        }
    }
}

/// Minimal standard base64, so the test needs no dependency to look for an encoded header.
fn base64(input: &str) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.as_bytes().chunks(3) {
        let n = chunk.iter().fold(0u32, |acc, b| (acc << 8) | *b as u32) << (8 * (3 - chunk.len()));
        for i in 0..=chunk.len() {
            out.push(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize] as char);
        }
        out.extend(std::iter::repeat_n('=', 3 - chunk.len()));
    }
    out
}
