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
    /// Uploads an activity file (FIT, TCX, GPX, ZIP or GZ) as
    /// `multipart/form-data`.
    ///
    /// The `name`, `description` and `external_id` fields are optional URL
    /// parameters documented by the API. The remote `filename` is derived
    /// from the local path.
    pub async fn upload_activity(
        &self,
        athlete_id: u64,
        file: &std::path::Path,
        name: Option<&str>,
        description: Option<&str>,
        external_id: Option<&str>,
    ) -> std::result::Result<ActivitySummary, oxisport_core::Error> {
        let path = format!("/athlete/{athlete_id}/activities");
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
