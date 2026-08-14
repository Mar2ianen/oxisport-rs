//! OxiSport shared async HTTP transport.
//!
//! `oxisport-runtime` provides the common mechanics so provider
//! contributors do not reimplement them: a configured [`Client`], request
//! execution, structured error classification, tracing hooks, and
//! streaming-friendly upload/download primitives.
//!
//! The runtime is Tokio-native: all network-facing APIs are asynchronous
//! and no Tokio runtime is ever created inside a library crate.
//!
//! Planned extension points (not yet implemented):
//! - OAuth and synchronized token refresh;
//! - pagination helpers;
//! - retry/backoff;
//! - rate limiting;
//! - multipart bodies;
//! - webhook verification;
//! - request concurrency limits.

pub mod body;
pub mod client;
pub mod response;
pub mod util;

pub use body::{ContentLength, MediaType, UploadBody};
pub use client::{Client, ClientBuilder, ClientConfig, RequestBuilder};
pub use response::{DownloadStream, Response};

pub use oxisport_core::{Error, RemoteError, Result};

/// Common HTTP types re-exported for convenience.
pub mod http {
    pub use http::header::{AUTHORIZATION, CONTENT_TYPE, RETRY_AFTER, USER_AGENT};
    pub use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
}
