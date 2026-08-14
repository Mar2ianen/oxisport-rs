//! The example provider: configuration, raw client access and capability
//! implementations.

use async_trait::async_trait;
use oxisport_core::{
    Activity, ActivityQuery, ActivitySource, ActivityStream, ProviderId, RemoteId, Result,
};
use oxisport_example_raw::generated::ExampleClient;
use oxisport_runtime::Client;
use url::Url;

use crate::mapping;

/// Default base URL of the mock service (unreachable by design).
pub const DEFAULT_BASE_URL: &str = "https://example.invalid";

/// Configuration for [`ExampleProvider`].
#[derive(Debug, Clone)]
pub struct ExampleConfig {
    /// Base URL of the service.
    pub base_url: String,
    /// Optional User-Agent override.
    pub user_agent: Option<String>,
}

impl Default for ExampleConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            user_agent: None,
        }
    }
}

/// The example (mock) provider.
///
/// Implements [`ActivitySource`] only; this is a demo of the capability
/// model, not a real service.
#[derive(Debug, Clone)]
pub struct ExampleProvider {
    raw: ExampleClient,
    provider: ProviderId,
}

impl ExampleProvider {
    /// Creates a provider from configuration.
    pub fn new(config: ExampleConfig) -> Result<Self> {
        let base_url = Url::parse(&config.base_url).map_err(|error| {
            oxisport_core::Error::invalid_request(format!(
                "invalid base URL '{}': {error}",
                config.base_url
            ))
        })?;
        let mut builder = Client::builder();
        if let Some(user_agent) = config.user_agent {
            builder = builder.user_agent(user_agent);
        }
        let client = builder.build()?;
        Ok(Self {
            raw: ExampleClient::new(client, base_url),
            provider: ProviderId::new("example"),
        })
    }

    /// Returns the provider identifier.
    pub fn provider_id(&self) -> &ProviderId {
        &self.provider
    }

    /// Returns the raw client for service-specific operations.
    pub fn raw(&self) -> &ExampleClient {
        &self.raw
    }
}

#[async_trait]
impl ActivitySource for ExampleProvider {
    async fn activity(&self, id: &RemoteId) -> Result<Activity> {
        let raw = self.raw.get_activity(id.as_str()).await?;
        Ok(mapping::activity(&self.provider, raw))
    }

    async fn activities(&self, _query: &ActivityQuery) -> Result<ActivityStream<'_>> {
        let raws = self.raw.list_activities().await?;
        let stream = futures_util::stream::iter(
            raws.into_iter()
                .map(|raw| Ok(mapping::activity(&self.provider, raw))),
        );
        Ok(Box::pin(stream))
    }
}
