//! Common error vocabulary.
//!
//! The categories here are shared across the whole framework. Provider
//! details (status code, response body, provider name) are preserved where
//! they are useful for debugging.

use std::fmt;
use std::time::Duration;

use thiserror::Error;

/// The common result type used across OxiSport.
pub type Result<T> = std::result::Result<T, Error>;

/// A common error across OxiSport.
///
/// Provider-specific details are preserved in [`Error::Remote`] via
/// [`RemoteError`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// A transport-level failure (DNS, connect, TLS, timeout, ...).
    #[error("transport error: {source}")]
    Transport {
        /// The underlying error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// Authentication is missing, invalid or expired.
    #[error("authentication failed: {0}")]
    Authentication(String),
    /// The identity is valid but lacks permission.
    #[error("authorization denied: {0}")]
    Authorization(String),
    /// The remote throttled the request.
    #[error("rate limited (retry after {retry_after:?})")]
    RateLimited {
        /// When the remote allows retrying, when available.
        retry_after: Option<Duration>,
    },
    /// The requested resource does not exist.
    #[error("not found: {0}")]
    NotFound(String),
    /// The request itself was malformed or rejected.
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    /// The remote reported a failure.
    #[error("remote error: {0}")]
    Remote(RemoteError),
    /// Serialization/deserialization failed.
    #[error("serialization error: {source}")]
    Serialization {
        /// The underlying error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// An I/O failure.
    #[error("io error: {source}")]
    Io {
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// The operation is not supported by the provider or the framework.
    #[error("unsupported: {0}")]
    Unsupported(String),
}

impl Error {
    /// Wraps a transport error.
    pub fn transport<E>(source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Transport {
            source: Box::new(source),
        }
    }

    /// Wraps a serialization error.
    pub fn serialization<E>(source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Serialization {
            source: Box::new(source),
        }
    }

    /// Creates an invalid-request error.
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest(message.into())
    }

    /// Creates an unsupported-operation error.
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::Unsupported(message.into())
    }

    /// Creates a rate-limited error.
    pub fn rate_limited(retry_after: Option<Duration>) -> Self {
        Self::RateLimited { retry_after }
    }

    /// Attaches a provider name to remote errors for easier debugging.
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        if let Self::Remote(remote) = &mut self {
            remote.provider = Some(provider.into());
        }
        self
    }

    /// Returns the provider name attached to a remote error, if any.
    pub fn provider(&self) -> Option<&str> {
        match self {
            Self::Remote(remote) => remote.provider.as_deref(),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Self {
        Self::Io { source }
    }
}

/// Details of a failure reported by the remote service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteError {
    /// Provider name, when attached by a provider or raw client.
    pub provider: Option<String>,
    /// HTTP status code, when available.
    pub status: Option<u16>,
    /// Response body excerpt, when available.
    pub body: Option<String>,
}

impl fmt::Display for RemoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.provider.as_deref(), self.status) {
            (Some(provider), Some(status)) => {
                write!(
                    f,
                    "{provider} returned status {status}: {}",
                    self.body_text()
                )
            }
            (Some(provider), None) => write!(f, "{provider}: {}", self.body_text()),
            (None, Some(status)) => {
                write!(f, "remote returned status {status}: {}", self.body_text())
            }
            (None, None) => write!(f, "remote error: {}", self.body_text()),
        }
    }
}

impl std::error::Error for RemoteError {}

impl RemoteError {
    fn body_text(&self) -> &str {
        self.body.as_deref().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Error, RemoteError};

    #[test]
    fn with_provider_annotates_remote_errors() {
        let err = Error::Remote(RemoteError {
            provider: None,
            status: Some(500),
            body: Some("boom".to_string()),
        })
        .with_provider("example");

        assert_eq!(err.provider(), Some("example"));
        assert!(err.to_string().contains("500"));
        assert!(err.to_string().contains("boom"));
    }

    #[test]
    fn with_provider_ignores_other_variants() {
        let err = Error::NotFound("x".to_string()).with_provider("example");
        assert_eq!(err.provider(), None);
    }

    #[test]
    fn helpers_build_variants() {
        assert!(matches!(
            Error::rate_limited(Some(Duration::from_secs(3))),
            Error::RateLimited { retry_after: Some(d) } if d == Duration::from_secs(3)
        ));
        assert!(matches!(
            Error::invalid_request("bad"),
            Error::InvalidRequest(_)
        ));
        assert!(matches!(Error::unsupported("nope"), Error::Unsupported(_)));
    }
}
