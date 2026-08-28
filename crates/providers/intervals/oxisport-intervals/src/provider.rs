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
pub const DEFAULT_ATHLETE_ID: &str = "0";

/// Configuration for [`IntervalsProvider`].
#[derive(Debug, Clone)]
pub struct IntervalsConfig {
    /// Personal API key (see <https://intervals.icu/settings>).
    pub api_key: String,
    /// Base URL of the API.
    pub base_url: String,
    /// Athlete id used in paths; `0` resolves to the API key's owner.
    pub athlete_id: String,
    /// Optional User-Agent override.
    pub user_agent: Option<String>,
}

impl Default for IntervalsConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            athlete_id: DEFAULT_ATHLETE_ID.to_string(),
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
    athlete_id: String,
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
        // Endpoint paths are joined relative to the base URL; a missing
        // trailing slash would replace the last path segment (e.g. `/api/v1`).
        let base_url = if base_url.path().ends_with('/') {
            base_url
        } else {
            let mut normalized = base_url;
            let path = format!("{}/", normalized.path());
            normalized.set_path(&path);
            normalized
        };
        let api_key = config.api_key.trim();
        let mut builder = Client::builder().basic_auth("API_KEY", api_key);
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
    pub fn athlete_id(&self) -> &str {
        &self.athlete_id
    }

    /// Returns the raw client for service-specific operations.
    pub fn raw(&self) -> &IntervalsClient {
        &self.raw
    }

    /// Returns the complete Intervals.icu athlete profile JSON.
    pub async fn athlete_full(&self) -> Result<serde_json::Value> {
        self.raw.get_athlete_full(&self.athlete_id).await
    }

    /// Returns wellness records scoped to the configured athlete.
    pub async fn wellness(
        &self,
        oldest: Option<&str>,
        newest: Option<&str>,
        cols: Option<&[String]>,
        fields: Option<&[String]>,
    ) -> Result<serde_json::Value> {
        self.raw
            .get_wellness(&self.athlete_id, oldest, newest, cols, fields)
            .await
    }

    /// Returns one wellness record scoped to the configured athlete.
    pub async fn wellness_day(&self, date: &str) -> Result<serde_json::Value> {
        self.raw.get_wellness_day(&self.athlete_id, date).await
    }

    /// Returns calendar events scoped to the configured athlete.
    pub async fn events(
        &self,
        oldest: Option<&str>,
        newest: Option<&str>,
        category: Option<&[String]>,
        limit: Option<i32>,
        resolve: bool,
    ) -> Result<serde_json::Value> {
        self.raw
            .list_events(&self.athlete_id, oldest, newest, category, limit, resolve)
            .await
    }

    /// Returns one calendar event scoped to the configured athlete.
    pub async fn event(&self, event_id: i64) -> Result<serde_json::Value> {
        self.raw.get_event(&self.athlete_id, event_id).await
    }

    /// Returns fitness-model events scoped to the configured athlete.
    pub async fn fitness_model_events(&self) -> Result<serde_json::Value> {
        self.raw.get_fitness_model_events(&self.athlete_id).await
    }

    /// Returns messages attached to an activity.
    pub async fn activity_messages(
        &self,
        activity_id: &str,
        since_id: Option<i64>,
        limit: Option<i32>,
    ) -> Result<serde_json::Value> {
        self.raw
            .get_activity_messages(activity_id, since_id, limit)
            .await
    }

    /// Returns the complete activity JSON from Intervals.icu.
    pub async fn activity_full(
        &self,
        id: &str,
        include_intervals: bool,
    ) -> Result<serde_json::Value> {
        self.raw.get_activity_full(id, include_intervals).await
    }

    /// Returns native CSV streams from Intervals.icu.
    pub async fn activity_streams_csv(
        &self,
        id: &str,
        types: Option<&[String]>,
        include_defaults: bool,
    ) -> Result<String> {
        self.raw
            .get_activity_streams_csv(id, types, include_defaults)
            .await
    }

    /// Downloads an original or converted activity file.
    pub async fn activity_file(&self, id: &str, format: &str) -> Result<Vec<u8>> {
        let (endpoint, include_power_hr) = match format {
            "fit" => ("fit-file", true),
            "gpx" => ("gpx-file", true),
            "original" => ("file", false),
            other => {
                return Err(oxisport_core::Error::invalid_request(format!(
                    "unsupported activity file format `{other}`; use fit, gpx or original"
                )));
            }
        };
        self.raw
            .get_activity_file(id, endpoint, include_power_hr)
            .await
    }

    /// Returns the provider's native bulk activity CSV for a date range.
    pub async fn activities_csv(
        &self,
        oldest: Option<&str>,
        newest: Option<&str>,
    ) -> Result<String> {
        let csv = self
            .raw
            .list_activities_csv(&self.athlete_id, oldest, newest)
            .await?;
        Self::filter_activity_csv(&csv, &self.athlete_id)
    }

    fn filter_activity_csv(csv_text: &str, athlete_id: &str) -> Result<String> {
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .from_reader(csv_text.trim_start_matches('\u{feff}').as_bytes());
        let headers = reader
            .headers()
            .map_err(oxisport_core::Error::serialization)?
            .clone();
        let athlete_column = headers
            .iter()
            .position(|header| header == "athlete_id")
            .ok_or_else(|| {
                oxisport_core::Error::invalid_request("bulk CSV has no athlete_id column")
            })?;
        let mut writer = csv::Writer::from_writer(Vec::new());
        writer
            .write_record(&headers)
            .map_err(oxisport_core::Error::serialization)?;
        for record in reader.records() {
            let record = record.map_err(oxisport_core::Error::serialization)?;
            if record.get(athlete_column) == Some(athlete_id) {
                writer
                    .write_record(&record)
                    .map_err(oxisport_core::Error::serialization)?;
            }
        }
        let bytes = writer
            .into_inner()
            .map_err(|error| oxisport_core::Error::serialization(error.into_error()))?;
        String::from_utf8(bytes).map_err(|error| {
            oxisport_core::Error::serialization(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                error,
            ))
        })
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
            .upload_activity(&self.athlete_id, file, name, description, external_id)
            .await?;
        Ok(mapping::activity(&self.provider, raw))
    }
}

#[async_trait]
impl AthleteSource for IntervalsProvider {
    async fn athlete(&self) -> Result<Athlete> {
        let raw = self.raw.get_athlete(&self.athlete_id).await?;
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
        // The Intervals.icu API requires the `oldest` parameter; without an
        // explicit lower bound use a date far in the past.
        let oldest = query
            .after
            .map(|time| time.date_naive().format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "2000-01-01".to_string());
        let newest = query
            .before
            .map(|time| time.date_naive().format("%Y-%m-%d").to_string());
        let raws = self
            .raw
            .list_activities(
                &self.athlete_id,
                Some(oldest.as_str()),
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

#[cfg(test)]
mod tests {
    use super::IntervalsProvider;

    #[test]
    fn bulk_csv_is_scoped_to_the_configured_athlete() {
        let input =
            "\u{feff}athlete_id,name,distance\ni282172,\"Long, ride\",325015\ni999,Other,10\n";
        let filtered =
            IntervalsProvider::filter_activity_csv(input, "i282172").expect("filters CSV");
        assert!(filtered.contains("i282172"));
        assert!(filtered.contains("Long, ride"));
        assert!(!filtered.contains("i999"));
    }
}
