//! Raw wire client for the Intervals.icu API.
//!
//! The `generated` module is produced by `oxisport-codegen` from
//! `specs/intervals/intervals.yaml` and committed to git. Do not edit it by
//! hand; regenerate with `cargo xtask codegen`.
//!
//! Endpoints that the generator does not cover (multipart uploads) are
//! implemented by hand below, still mirroring the wire API only: no
//! normalization and no knowledge of `oxisport-core` entities. Mapping into
//! the normalized model happens in the `oxisport-intervals` adapter.
//!
//! API source: https://intervals.icu/api/v1/docs

pub mod generated;

pub use generated::*;

impl IntervalsClient {
    async fn get_json(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> std::result::Result<serde_json::Value, oxisport_core::Error> {
        let mut url = self.base_url.join(path).map_err(|e| {
            oxisport_core::Error::invalid_request(format!("invalid request URL: {e}"))
        })?;
        for (key, value) in query {
            url.query_pairs_mut().append_pair(key, value);
        }
        let response = self
            .client
            .request(oxisport_runtime::http::Method::GET, url)
            .send()
            .await
            .map_err(|e| e.with_provider("intervals"))?;
        response.json().await
    }

    /// Returns wellness records for the configured athlete and date range.
    pub async fn get_wellness(
        &self,
        athlete_id: &str,
        oldest: Option<&str>,
        newest: Option<&str>,
        cols: Option<&[String]>,
        fields: Option<&[String]>,
    ) -> std::result::Result<serde_json::Value, oxisport_core::Error> {
        let mut query = Vec::new();
        if let Some(value) = oldest {
            query.push(("oldest".to_string(), value.to_string()));
        }
        if let Some(value) = newest {
            query.push(("newest".to_string(), value.to_string()));
        }
        if let Some(values) = cols.filter(|values| !values.is_empty()) {
            query.push(("cols".to_string(), values.join(",")));
        }
        if let Some(values) = fields.filter(|values| !values.is_empty()) {
            query.push(("fields".to_string(), values.join(",")));
        }
        self.get_json(&format!("athlete/{athlete_id}/wellness"), &query)
            .await
    }

    /// Returns one wellness record for the configured athlete.
    pub async fn get_wellness_day(
        &self,
        athlete_id: &str,
        date: &str,
    ) -> std::result::Result<serde_json::Value, oxisport_core::Error> {
        self.get_json(&format!("athlete/{athlete_id}/wellness/{date}"), &[])
            .await
    }

    /// Returns calendar events for the configured athlete and date range.
    pub async fn list_events(
        &self,
        athlete_id: &str,
        oldest: Option<&str>,
        newest: Option<&str>,
        category: Option<&[String]>,
        limit: Option<i32>,
        resolve: bool,
    ) -> std::result::Result<serde_json::Value, oxisport_core::Error> {
        let mut query = vec![("resolve".to_string(), resolve.to_string())];
        if let Some(value) = oldest {
            query.push(("oldest".to_string(), value.to_string()));
        }
        if let Some(value) = newest {
            query.push(("newest".to_string(), value.to_string()));
        }
        if let Some(values) = category.filter(|values| !values.is_empty()) {
            query.push(("category".to_string(), values.join(",")));
        }
        if let Some(value) = limit {
            query.push(("limit".to_string(), value.to_string()));
        }
        self.get_json(&format!("athlete/{athlete_id}/events"), &query)
            .await
    }

    /// Returns one calendar event.
    pub async fn get_event(
        &self,
        athlete_id: &str,
        event_id: i64,
    ) -> std::result::Result<serde_json::Value, oxisport_core::Error> {
        self.get_json(&format!("athlete/{athlete_id}/events/{event_id}"), &[])
            .await
    }

    /// Returns fitness-model events for the configured athlete.
    pub async fn get_fitness_model_events(
        &self,
        athlete_id: &str,
    ) -> std::result::Result<serde_json::Value, oxisport_core::Error> {
        self.get_json(&format!("athlete/{athlete_id}/fitness-model-events"), &[])
            .await
    }

    /// Returns messages attached to an activity.
    pub async fn get_activity_messages(
        &self,
        activity_id: &str,
        since_id: Option<i64>,
        limit: Option<i32>,
    ) -> std::result::Result<serde_json::Value, oxisport_core::Error> {
        let mut query = Vec::new();
        if let Some(value) = since_id {
            query.push(("sinceId".to_string(), value.to_string()));
        }
        if let Some(value) = limit {
            query.push(("limit".to_string(), value.to_string()));
        }
        self.get_json(&format!("activity/{activity_id}/messages"), &query)
            .await
    }

    /// Returns the complete Intervals.icu athlete profile JSON.
    pub async fn get_athlete_full(
        &self,
        id: &str,
    ) -> std::result::Result<serde_json::Value, oxisport_core::Error> {
        let path = format!("athlete/{id}");
        let url = self.base_url.join(&path).map_err(|e| {
            oxisport_core::Error::invalid_request(format!("invalid request URL: {e}"))
        })?;
        let response = self
            .client
            .request(oxisport_runtime::http::Method::GET, url)
            .send()
            .await
            .map_err(|e| e.with_provider("intervals"))?;
        response.json().await
    }

    /// Returns the complete Intervals.icu activity JSON, optionally including
    /// detected intervals.
    pub async fn get_activity_full(
        &self,
        id: &str,
        include_intervals: bool,
    ) -> std::result::Result<serde_json::Value, oxisport_core::Error> {
        let path = format!("activity/{id}");
        let url = self.base_url.join(&path).map_err(|e| {
            oxisport_core::Error::invalid_request(format!("invalid request URL: {e}"))
        })?;
        let request = self
            .client
            .request(oxisport_runtime::http::Method::GET, url)
            .query(&[("intervals", include_intervals)]);
        let response = request
            .send()
            .await
            .map_err(|e| e.with_provider("intervals"))?;
        response.json().await
    }

    /// Returns native CSV activity streams without normalizing or losing data.
    pub async fn get_activity_streams_csv(
        &self,
        id: &str,
        types: Option<&[String]>,
        include_defaults: bool,
    ) -> std::result::Result<String, oxisport_core::Error> {
        let path = format!("activity/{id}/streams.csv");
        let mut url = self.base_url.join(&path).map_err(|e| {
            oxisport_core::Error::invalid_request(format!("invalid request URL: {e}"))
        })?;
        if let Some(types) = types.filter(|types| !types.is_empty()) {
            url.query_pairs_mut().append_pair("types", &types.join(","));
        }
        url.query_pairs_mut()
            .append_pair("includeDefaults", &include_defaults.to_string());
        let response = self
            .client
            .request(oxisport_runtime::http::Method::GET, url)
            .send()
            .await
            .map_err(|e| e.with_provider("intervals"))?;
        let body = response.bytes().await?;
        String::from_utf8(body.to_vec()).map_err(|error| {
            oxisport_core::Error::serialization(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                error,
            ))
        })
    }

    /// Downloads an original or converted activity file.
    pub async fn get_activity_file(
        &self,
        id: &str,
        endpoint: &str,
        include_power_hr: bool,
    ) -> std::result::Result<Vec<u8>, oxisport_core::Error> {
        let path = format!("activity/{id}/{endpoint}");
        let mut url = self.base_url.join(&path).map_err(|e| {
            oxisport_core::Error::invalid_request(format!("invalid request URL: {e}"))
        })?;
        if include_power_hr {
            url.query_pairs_mut()
                .append_pair("power", "true")
                .append_pair("hr", "true");
        }
        let response = self
            .client
            .request(oxisport_runtime::http::Method::GET, url)
            .send()
            .await
            .map_err(|e| e.with_provider("intervals"))?;
        Ok(response.bytes().await?.to_vec())
    }

    /// Returns the provider's native bulk activity CSV for a date range.
    pub async fn list_activities_csv(
        &self,
        athlete_id: &str,
        oldest: Option<&str>,
        newest: Option<&str>,
    ) -> std::result::Result<String, oxisport_core::Error> {
        let path = format!("athlete/{athlete_id}/athlete-activities.csv");
        let mut url = self.base_url.join(&path).map_err(|e| {
            oxisport_core::Error::invalid_request(format!("invalid request URL: {e}"))
        })?;
        if let Some(oldest) = oldest {
            url.query_pairs_mut().append_pair("oldest", oldest);
        }
        if let Some(newest) = newest {
            url.query_pairs_mut().append_pair("newest", newest);
        }
        let response = self
            .client
            .request(oxisport_runtime::http::Method::GET, url)
            .send()
            .await
            .map_err(|e| e.with_provider("intervals"))?;
        let body = response.bytes().await?;
        String::from_utf8(body.to_vec()).map_err(|error| {
            oxisport_core::Error::serialization(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                error,
            ))
        })
    }

    /// Uploads an activity file (FIT, TCX, GPX, ZIP or GZ) as
    /// `multipart/form-data`.
    ///
    /// The `name`, `description` and `external_id` fields are optional URL
    /// parameters documented by the API. The remote `filename` is derived
    /// from the local path.
    pub async fn upload_activity(
        &self,
        athlete_id: &str,
        file: &std::path::Path,
        name: Option<&str>,
        description: Option<&str>,
        external_id: Option<&str>,
    ) -> std::result::Result<ActivitySummary, oxisport_core::Error> {
        let path = format!("athlete/{athlete_id}/activities");
        let url = self.base_url.join(&path).map_err(|e| {
            oxisport_core::Error::invalid_request(format!("invalid request URL: {e}"))
        })?;

        let form = oxisport_runtime::MultipartForm::new()
            .file("file", file)
            .await
            .map_err(|e| {
                oxisport_core::Error::invalid_request(format!("cannot open upload file: {e}"))
            })?;

        let mut request = self.client.post(url).multipart(form);
        let mut query: Vec<(&str, &str)> = Vec::new();
        if let Some(name) = name {
            query.push(("name", name));
        }
        if let Some(description) = description {
            query.push(("description", description));
        }
        if let Some(external_id) = external_id {
            query.push(("external_id", external_id));
        }
        if !query.is_empty() {
            request = request.query(&query);
        }

        let response = request
            .send()
            .await
            .map_err(|e| e.with_provider("intervals"))?;
        response.json().await
    }
}
