//! Response handling and streaming downloads.

use bytes::Bytes;
use futures_core::Stream;
use futures_util::StreamExt;
use http::HeaderMap;
use http::StatusCode;
use serde::de::DeserializeOwned;

use crate::body::MediaType;
use crate::{ContentLength, Error, Result};

/// A received HTTP response.
///
/// Error statuses are classified into [`Error`] variants before a
/// `Response` is handed out, so successful responses carry only 2xx
/// statuses.
#[derive(Debug)]
pub struct Response {
    inner: reqwest::Response,
}

impl Response {
    pub(crate) async fn from_reqwest(response: reqwest::Response) -> Result<Self> {
        let status = response.status();
        if status.is_success() {
            return Ok(Self { inner: response });
        }
        let headers = response.headers().clone();
        let body = response.text().await.unwrap_or_default();
        let body = truncate(&body, 4096);
        Err(classify(status, &body, &headers))
    }

    /// Returns the response status.
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    /// Returns the response headers.
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    /// Deserializes the response body as JSON.
    pub async fn json<T: DeserializeOwned>(self) -> Result<T> {
        let body = self.inner.bytes().await.map_err(Error::transport)?;
        serde_json::from_slice(&body).map_err(|error| {
            let preview = String::from_utf8_lossy(&body[..body.len().min(200)]);
            Error::serialization(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("{error}; response prefix: {preview:?}"),
            ))
        })
    }

    /// Buffers the whole response body into memory.
    ///
    /// Prefer [`download`](Response::download) for large payloads.
    pub async fn bytes(self) -> Result<Bytes> {
        self.inner.bytes().await.map_err(Error::transport)
    }

    /// Converts the response into a streaming download.
    pub fn download(self) -> DownloadStream {
        DownloadStream::new(self.inner)
    }
}

/// A streaming download with metadata.
///
/// The body can be consumed as an async byte stream without buffering the
/// entire payload in memory.
#[derive(Debug)]
pub struct DownloadStream {
    inner: reqwest::Response,
    content_length: ContentLength,
    media_type: Option<MediaType>,
}

impl DownloadStream {
    fn new(response: reqwest::Response) -> Self {
        let content_length = response
            .headers()
            .get(http::header::CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .map(ContentLength::new)
            .unwrap_or_else(ContentLength::unknown);
        let media_type =
            MediaType::from_content_type(response.headers().get(http::header::CONTENT_TYPE));
        Self {
            inner: response,
            content_length,
            media_type,
        }
    }

    /// Returns the content length, when the remote announced one.
    pub fn content_length(&self) -> ContentLength {
        self.content_length
    }

    /// Returns the media type, when the remote announced one.
    pub fn media_type(&self) -> Option<&MediaType> {
        self.media_type.as_ref()
    }

    /// Returns the response status.
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    /// Returns the response headers.
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    /// Consumes the download as an async byte stream.
    pub fn into_stream(self) -> impl Stream<Item = Result<Bytes>> + Send {
        self.inner
            .bytes_stream()
            .map(|chunk| chunk.map_err(Error::transport))
    }

    /// Buffers the entire download into memory.
    ///
    /// Provided for convenience and for APIs that genuinely require the
    /// whole payload; prefer [`into_stream`](DownloadStream::into_stream)
    /// for large files.
    pub async fn into_bytes(self) -> Result<Bytes> {
        let mut buffer = Vec::new();
        let mut stream = self.into_stream();
        while let Some(chunk) = stream.next().await {
            buffer.extend_from_slice(&chunk?);
        }
        Ok(Bytes::from(buffer))
    }
}

/// Maps an HTTP status into a structured error.
fn classify(status: StatusCode, body: &str, headers: &HeaderMap) -> Error {
    match status {
        StatusCode::UNAUTHORIZED => Error::Authentication(format!("status {status}: {body}")),
        StatusCode::FORBIDDEN => Error::Authorization(format!("status {status}: {body}")),
        StatusCode::NOT_FOUND => Error::NotFound(body.to_string()),
        StatusCode::TOO_MANY_REQUESTS => Error::RateLimited {
            retry_after: retry_after(headers),
        },
        status if status.is_client_error() => {
            Error::InvalidRequest(format!("status {status}: {body}"))
        }
        status => Error::Remote(remote_error(status, body)),
    }
}

fn remote_error(status: StatusCode, body: &str) -> crate::RemoteError {
    crate::RemoteError {
        provider: None,
        status: Some(status.as_u16()),
        body: Some(body.to_string()),
    }
}

/// Parses `Retry-After` in the delta-seconds form.
fn retry_after(headers: &HeaderMap) -> Option<std::time::Duration> {
    let value = headers.get(http::header::RETRY_AFTER)?.to_str().ok()?;
    let seconds = value.trim().parse::<u64>().ok()?;
    Some(std::time::Duration::from_secs(seconds))
}

fn truncate(value: &str, max: usize) -> String {
    if value.len() <= max {
        value.to_string()
    } else {
        let mut truncated = value.chars().take(max).collect::<String>();
        truncated.push_str("...");
        truncated
    }
}
