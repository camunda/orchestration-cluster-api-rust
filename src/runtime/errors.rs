//! Typed error surface for the SDK, mirroring the JS/Python/C# SDKs' error taxonomy.

use std::fmt;

/// Convenience alias used throughout the SDK.
pub type Result<T> = std::result::Result<T, CamundaError>;

/// Errors returned by the Camunda SDK.
#[derive(Debug, thiserror::Error)]
pub enum CamundaError {
    /// Configuration was invalid or incomplete (e.g. missing OAuth credentials).
    #[error("configuration error: {0}")]
    Config(String),

    /// Failed to obtain or refresh an authentication token.
    #[error("authentication error: {0}")]
    Auth(String),

    /// A network-level failure occurred (connection, TLS, timeout).
    #[error("network error: {0}")]
    Network(#[source] reqwest::Error),

    /// An I/O failure occurred (e.g. while streaming a multipart upload).
    #[error("I/O error: {0}")]
    Io(#[source] std::io::Error),

    /// The server returned a non-success HTTP status.
    #[error("API error: HTTP {status}{}", .body.as_ref().map(|b| format!(" — {b}")).unwrap_or_default())]
    Api {
        /// HTTP status code.
        status: u16,
        /// Response body, if any (often an RFC 7807 problem detail).
        body: Option<String>,
    },

    /// A response payload could not be (de)serialized.
    #[error("serialization error: {0}")]
    Serialization(#[source] serde_json::Error),

    /// A domain-type or input constraint was violated before sending the request.
    #[error("validation error: {0}")]
    Validation(String),
}

impl CamundaError {
    pub(crate) fn config(msg: impl Into<String>) -> Self {
        CamundaError::Config(msg.into())
    }

    pub(crate) fn auth(msg: impl Into<String>) -> Self {
        CamundaError::Auth(msg.into())
    }

    /// The HTTP status code, if this is a [`CamundaError::Api`] error.
    pub fn status(&self) -> Option<u16> {
        match self {
            CamundaError::Api { status, .. } => Some(*status),
            _ => None,
        }
    }
}

/// Convert a generated-client error into a [`CamundaError`].
///
/// The generated operations return `Error<T>` where `T` is the operation-specific
/// error enum. We collapse that into the SDK's typed taxonomy.
impl<T: fmt::Debug> From<camunda_orchestration_api_client::apis::Error<T>> for CamundaError {
    fn from(err: camunda_orchestration_api_client::apis::Error<T>) -> Self {
        use camunda_orchestration_api_client::apis::Error as GenError;
        match err {
            GenError::Reqwest(e) => CamundaError::Network(e),
            GenError::Serde(e) => CamundaError::Serialization(e),
            GenError::Io(e) => CamundaError::Io(e),
            GenError::ResponseError(content) => CamundaError::Api {
                status: content.status.as_u16(),
                body: Some(content.content),
            },
        }
    }
}

impl From<reqwest::Error> for CamundaError {
    fn from(e: reqwest::Error) -> Self {
        CamundaError::Network(e)
    }
}

impl From<serde_json::Error> for CamundaError {
    fn from(e: serde_json::Error) -> Self {
        CamundaError::Serialization(e)
    }
}
