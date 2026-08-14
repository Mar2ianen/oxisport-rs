//! The Intervals.icu provider: configuration, raw client access and
//! capability implementations.

use async_trait::async_trait;
use oxisport_core::{
    Activity, ActivityQuery, ActivitySource, ActivityStream, Athlete, AthleteSource, ProviderId,
    RemoteId, Result,
};
use oxisport_intervals_raw::{ActivitySummary, IntervalsClient};
use oxisport_runtime::Client;
use url::Url;

use crate::mapping;

/// Default base URL of the Intervals.icu API.
pub const DEFAULT_BASE_URL: &str = "https://intervals.icu/api/v1";

/// Athlete id `0` means "the athlete the API key belongs to".
pub const DEFAULT_ATHLETE_ID: u64 = 0;

/// Configuration for [`IntervalsProvider`].
#[derive(Debug, Clone)]
pub struct IntervalsConfig {
    /// Personal API key (see <https://intervals.icu/settings>).
    pub api_key: String,
    /// Base URL of the API.
    pub base_url: String,
    /// Athlete id used in paths; `0` resolves to the API key's owner.
    pub athlete_id: u64,
    /// Optional User-Agent override.
    pub user_agent: Option<String>,
}

impl Default for IntervalsConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            athlete_id: DEFAULT_ATHLETE_ID,
            user_agent: None,
        }
    }
}

/// The Intervals.icu provider.
///
/// Implements [`AthleteSource`] and [`ActivitySource`].
#[derive(Debug, Clone)]
pub struct IntervalsProvider {
    raw: IntervalsClient,
    provider: ProviderId,
    athlete_id: u64,
}

impl IntervalsProvider {
    /// Creates a provider from configuration.
    pub fn new(config: IntervalsConfig) -> Result<Self> {
        if config.api_key.trim().is_empty() {
            return Err(oxisport_core::Error::invalid_request(
                "an Intervals.icu API key is required (https://intervals.icu/settings)",
            ));
        }
        let base_url = Url::parse(&config.base_url).map_err(|error| {
            oxisport_core::Error::invalid_request(format!(
                "invalid base URL '{}': {error}",
                config.base_url
            ))
        })?;
        let api_key = oxisport_runtime::http::HeaderValue::from_str(config.api_key.trim())
            .map_err(|error| {
                oxisport_core::Error::invalid_request(format!("invalid API key: {error}"))
            })?;
        let mut builder = Client::builder().default_header(
            oxisport_runtime::http::HeaderName::from_static("x-api-key"),
            api_key,
        );
        if let Some(user_agent) = config.user_agent {
            builder = builder.user_agent(user_agent);
        }
        let client = builder.build()?;
        Ok(Self {
            raw: IntervalsClient::new(client, base_url),
            provider: ProviderId::new("intervals"),
            athlete_id: config.athlete_id,
        })
    }

    /// Returns the provider identifier.
    pub fn provider_id(&self) -> &ProviderId {
        &self.provider
    }

    /// Returns the athlete id used in API paths.
    pub fn athlete_id(&self) -> u64 {
        self.athlete_id
    }

    /// Returns the raw client for service-specific operations.
    pub fn raw(&self) -> &IntervalsClient {
        &self.raw
    }

    /// Uploads an activity file (FIT, TCX, GPX, ZIP or GZ).
    ///
    /// `name`, `description` and `external_id` are optional metadata sent
    /// with the upload. Returns the stored activity.
    pub async fn upload_activity(
        &self,
        file: &std::path::Path,
        name: Option<&str>,
        description: Option<&str>,
        external_id: Option<&str>,
    ) -> Result<Activity> {
        let raw: ActivitySummary = self
            .raw
            .upload_activity(self.athlete_id, file, name, description, external_id)
            .await?;
        Ok(mapping::activity(&self.provider, raw))
    }
}

#[async_trait]
impl AthleteSource for IntervalsProvider {
    async fn athlete(&self) -> Result<Athlete> {
        let raw = self.raw.get_athlete(self.athlete_id).await?;
        Ok(mapping::athlete(&self.provider, raw))
    }
}

#[async_trait]
impl ActivitySource for IntervalsProvider {
    async fn activity(&self, id: &RemoteId) -> Result<Activity> {
        let raw = self.raw.get_activity(id.as_str()).await?;
        Ok(mapping::activity(&self.provider, raw))
    }

    async fn activities(&self, query: &ActivityQuery) -> Result<ActivityStream<'_>> {
        let oldest = query
            .after
            .map(|time| time.date_naive().format("%Y-%m-%d").to_string());
        let newest = query
            .before
            .map(|time| time.date_naive().format("%Y-%m-%d").to_string());
        let raws = self
            .raw
            .list_activities(
                self.athlete_id,
                oldest.as_deref(),
                newest.as_deref(),
                query.limit,
            )
            .await?;
        let stream = futures_util::stream::iter(
            raws.into_iter()
                .map(|raw| Ok(mapping::activity(&self.provider, raw))),
        );
        Ok(Box::pin(stream))
    }
}
