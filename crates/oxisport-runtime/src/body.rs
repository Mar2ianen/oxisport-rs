//! Streaming-friendly upload bodies.

use std::io;
use std::path::PathBuf;
use std::pin::Pin;

use bytes::Bytes;
use futures_core::Stream;
use http::HeaderValue;
use tokio_util::io::ReaderStream;

/// An upload body that avoids forcing whole-file buffering.
///
/// Supports in-memory [`Bytes`], a filesystem [`PathBuf`], or an async byte
/// stream. The stream variant is sent chunked, so large files can be
/// transferred without loading them entirely into memory.
pub enum UploadBody {
    /// A body already held in memory.
    Bytes(Bytes),
    /// A body to be streamed from a file on disk, opened lazily at send time.
    File(PathBuf),
    /// An arbitrary async byte stream.
    Stream(Pin<Box<dyn Stream<Item = io::Result<Bytes>> + Send>>),
}

impl UploadBody {
    /// Creates a body from in-memory bytes.
    pub fn from_bytes(bytes: impl Into<Bytes>) -> Self {
        Self::Bytes(bytes.into())
    }

    /// Creates a body that streams a file from disk.
    pub fn from_file(path: impl Into<PathBuf>) -> Self {
        Self::File(path.into())
    }

    /// Creates a body from an async byte stream.
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: Stream<Item = io::Result<Bytes>> + Send + 'static,
    {
        Self::Stream(Box::pin(stream))
    }

    /// Returns the known content length, if the body size is known up front.
    ///
    /// File and stream bodies report unknown length; file bodies are sent
    /// chunked by the underlying HTTP stack.
    pub fn content_length(&self) -> crate::ContentLength {
        match self {
            UploadBody::Bytes(bytes) => crate::ContentLength::new(bytes.len() as u64),
            UploadBody::File(_) | UploadBody::Stream(_) => crate::ContentLength::unknown(),
        }
    }

    /// Converts into a `reqwest::Body`.
    pub fn into_reqwest_body(self) -> reqwest::Body {
        match self {
            UploadBody::Bytes(bytes) => reqwest::Body::from(bytes),
            UploadBody::File(path) => file_body(path),
            UploadBody::Stream(stream) => reqwest::Body::wrap_stream(stream),
        }
    }
}

impl From<Bytes> for UploadBody {
    fn from(value: Bytes) -> Self {
        Self::Bytes(value)
    }
}

impl From<Vec<u8>> for UploadBody {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(Bytes::from(value))
    }
}

impl From<&'static [u8]> for UploadBody {
    fn from(value: &'static [u8]) -> Self {
        Self::Bytes(Bytes::from_static(value))
    }
}

impl From<PathBuf> for UploadBody {
    fn from(value: PathBuf) -> Self {
        Self::File(value)
    }
}

impl From<&std::path::Path> for UploadBody {
    fn from(value: &std::path::Path) -> Self {
        Self::File(value.to_path_buf())
    }
}

/// Streams a file lazily: the file is opened on the first poll, so errors
/// surface through the request instead of at body construction time.
fn file_body(path: PathBuf) -> reqwest::Body {
    use futures_util::TryStreamExt;

    let opened = futures_util::stream::once(async move { tokio::fs::File::open(path).await });
    let stream: Pin<Box<dyn Stream<Item = io::Result<Bytes>> + Send>> =
        Box::pin(opened.map_ok(ReaderStream::new).try_flatten());
    reqwest::Body::wrap_stream(stream)
}

/// An HTTP content length, possibly unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentLength(Option<u64>);

impl ContentLength {
    /// Creates a known content length.
    pub const fn new(length: u64) -> Self {
        Self(Some(length))
    }

    /// Creates an unknown content length.
    pub const fn unknown() -> Self {
        Self(None)
    }

    /// Returns the length when known.
    pub const fn get(&self) -> Option<u64> {
        self.0
    }
}

impl From<u64> for ContentLength {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Display for ContentLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(length) => write!(f, "{length}"),
            None => f.write_str("unknown"),
        }
    }
}

/// A media type (MIME type), e.g. `application/json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaType(String);

impl MediaType {
    /// Creates a media type.
    pub fn new(media_type: impl Into<String>) -> Self {
        Self(media_type.into())
    }

    /// Returns the media type as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Parses a media type from a `Content-Type` header value, stripping
    /// parameters such as `charset=...`.
    pub fn from_content_type(value: Option<&HeaderValue>) -> Option<Self> {
        let raw = value?.to_str().ok()?;
        let media_type = raw.split(';').next()?.trim();
        if media_type.is_empty() {
            None
        } else {
            Some(Self(media_type.to_string()))
        }
    }
}

impl From<&str> for MediaType {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for MediaType {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
