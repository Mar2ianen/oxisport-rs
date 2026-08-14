//! Shared reqwest-based HTTP client.

use std::time::Duration;

use http::{HeaderMap, HeaderName, HeaderValue, Method};
use serde::Serialize;
use url::Url;

use crate::body::{MediaType, MultipartForm, UploadBody};
use crate::response::Response;
use crate::{Error, Result};

/// Configuration for a [`Client`].
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// User-Agent sent with every request.
    pub user_agent: String,
    /// Connect timeout.
    pub connect_timeout: Option<Duration>,
    /// Per-request timeout.
    pub timeout: Option<Duration>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            user_agent: format!("oxisport-runtime/{}", env!("CARGO_PKG_VERSION")),
            connect_timeout: Some(Duration::from_secs(10)),
            timeout: Some(Duration::from_secs(60)),
        }
    }
}

/// A configured HTTP client.
///
/// Cheap to clone; all instances share the underlying connection pool.
#[derive(Debug, Clone)]
pub struct Client {
    inner: reqwest::Client,
    config: ClientConfig,
}

impl Client {
    /// Starts building a client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Returns the effective configuration.
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Starts a request with the given method and URL.
    pub fn request(&self, method: Method, url: Url) -> RequestBuilder {
        let request = self.inner.request(method.clone(), url.clone());
        RequestBuilder {
            method,
            url,
            request,
            headers: HeaderMap::new(),
            media_type: None,
        }
    }

    /// Starts a GET request.
    pub fn get(&self, url: Url) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    /// Starts a POST request.
    pub fn post(&self, url: Url) -> RequestBuilder {
        self.request(Method::POST, url)
    }

    /// Starts a PUT request.
    pub fn put(&self, url: Url) -> RequestBuilder {
        self.request(Method::PUT, url)
    }

    /// Starts a DELETE request.
    pub fn delete(&self, url: Url) -> RequestBuilder {
        self.request(Method::DELETE, url)
    }

    /// Starts a PATCH request.
    pub fn patch(&self, url: Url) -> RequestBuilder {
        self.request(Method::PATCH, url)
    }
}

/// Builds a [`Client`].
#[derive(Debug)]
pub struct ClientBuilder {
    config: ClientConfig,
    default_headers: HeaderMap,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            config: ClientConfig::default(),
            default_headers: HeaderMap::new(),
        }
    }
}

impl ClientBuilder {
    /// Overrides the User-Agent.
    #[must_use]
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = user_agent.into();
        self
    }

    /// Sets the connect timeout.
    #[must_use]
    pub fn connect_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    /// Sets the per-request timeout.
    #[must_use]
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Adds a default header sent with every request.
    #[must_use]
    pub fn default_header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.default_headers.insert(name, value);
        self
    }

    /// Builds the client.
    pub fn build(self) -> Result<Client> {
        let mut builder = reqwest::Client::builder()
            .user_agent(&self.config.user_agent)
            .default_headers(self.default_headers);
        if let Some(timeout) = self.config.connect_timeout {
            builder = builder.connect_timeout(timeout);
        }
        if let Some(timeout) = self.config.timeout {
            builder = builder.timeout(timeout);
        }
        let inner = builder.build().map_err(Error::transport)?;
        Ok(Client {
            inner,
            config: self.config,
        })
    }
}

/// An in-progress request.
///
/// Builder methods return `Self` and must be chained; nothing is sent until
/// [`send`](RequestBuilder::send) is awaited.
pub struct RequestBuilder {
    method: Method,
    url: Url,
    request: reqwest::RequestBuilder,
    headers: HeaderMap,
    media_type: Option<MediaType>,
}

impl RequestBuilder {
    /// Appends a query string serialized from `query`.
    #[must_use]
    pub fn query<T: Serialize + ?Sized>(mut self, query: &T) -> Self {
        self.request = self.request.query(query);
        self
    }

    /// Sets a request header.
    #[must_use]
    pub fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.headers.insert(name, value);
        self
    }

    /// Sets multiple request headers.
    #[must_use]
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers.extend(headers);
        self
    }

    /// Serializes `value` as a JSON request body.
    #[must_use]
    pub fn json<T: Serialize + ?Sized>(mut self, value: &T) -> Self {
        self.media_type = Some(MediaType::new("application/json"));
        self.request = self.request.json(value);
        self
    }

    /// Sets an upload body.
    #[must_use]
    pub fn body<B: Into<UploadBody>>(mut self, body: B) -> Self {
        let body: UploadBody = body.into();
        self.request = self.request.body(body.into_reqwest_body());
        self
    }

    /// Sets a `multipart/form-data` body.
    ///
    /// The `Content-Type` (including the boundary) is generated by the
    /// underlying HTTP stack; do not combine with [`json`](Self::json) or
    /// [`body`](Self::body).
    #[must_use]
    pub fn multipart(mut self, form: MultipartForm) -> Self {
        self.request = self.request.multipart(form.into_reqwest_form());
        self
    }

    /// Sets a `Bearer` authorization header from `token`.
    #[must_use]
    pub fn bearer_token(mut self, token: &str) -> Self {
        if let Ok(value) = HeaderValue::from_str(&format!("Bearer {token}")) {
            self.headers.insert(http::header::AUTHORIZATION, value);
        } else {
            tracing::warn!("bearer token contains invalid characters; header omitted");
        }
        self
    }

    /// Sends the request and classifies the response status.
    pub async fn send(self) -> Result<Response> {
        let mut request = self.request;
        for (name, value) in &self.headers {
            request = request.header(name, value);
        }
        if let Some(media_type) = &self.media_type {
            request = request.header(http::header::CONTENT_TYPE, media_type.as_str());
        }
        tracing::debug!(method = %self.method, url = %self.url, "sending request");
        let response = request.send().await.map_err(Error::transport)?;
        tracing::debug!(status = %response.status(), "request completed");
        Response::from_reqwest(response).await
    }
}
