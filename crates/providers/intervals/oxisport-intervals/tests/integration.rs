//! Integration tests for the Intervals.icu provider using wiremock.
//!
//! No credentials are needed: the API key header is asserted but the mock
//! accepts any value.

use futures_util::TryStreamExt;
use oxisport_core::{
    ActivityQuery, ActivitySource, AthleteSource, Error, ProviderId, RemoteId, Sport,
};
use oxisport_intervals::{IntervalsConfig, IntervalsProvider};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn provider_for(server: &MockServer) -> IntervalsProvider {
    IntervalsProvider::new(IntervalsConfig {
        api_key: "test-api-key".to_string(),
        base_url: server.uri(),
        ..IntervalsConfig::default()
    })
    .expect("provider builds")
}

const ACTIVITY_JSON: &str = r#"{
    "id": "i55610271",
    "name": "Morning ride",
    "type": "Ride",
    "start_date_local": "2026-08-14T06:30:00",
    "distance": 45000.0,
    "moving_time": 5400,
    "average_heartrate": 139,
    "average_watts": 213,
    "average_cadence": 88
}"#;

fn activity_value() -> serde_json::Value {
    serde_json::from_str(ACTIVITY_JSON).expect("activity fixture parses")
}

#[tokio::test]
async fn missing_api_key_is_rejected() {
    let error = IntervalsProvider::new(IntervalsConfig::default()).expect_err("rejected");
    assert!(matches!(error, Error::InvalidRequest(_)));
}

#[tokio::test]
async fn fetches_athlete_with_api_key_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/athlete/0"))
        .and(header("x-api-key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 2049151,
            "first_name": "John",
            "last_name": "Doe"
        })))
        .mount(&server)
        .await;

    let athlete = provider_for(&server)
        .athlete()
        .await
        .expect("fetches athlete");

    assert_eq!(athlete.id.as_str(), "2049151");
    assert_eq!(athlete.provider, ProviderId::new("intervals"));
    assert_eq!(athlete.name.as_deref(), Some("John Doe"));
}

#[tokio::test]
async fn lists_activities_with_date_range_and_limit() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/athlete/0/activities"))
        .and(query_param("oldest", "2026-01-01"))
        .and(query_param("newest", "2026-02-01"))
        .and(query_param("limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![activity_value()]))
        .mount(&server)
        .await;

    let provider = provider_for(&server);
    let query = ActivityQuery {
        after: Some("2026-01-01T00:00:00Z".parse().unwrap()),
        before: Some("2026-02-01T00:00:00Z".parse().unwrap()),
        limit: Some(5),
    };
    let activities = provider
        .activities(&query)
        .await
        .expect("lists activities")
        .try_collect::<Vec<_>>()
        .await
        .expect("stream succeeds");

    assert_eq!(activities.len(), 1);
    assert_eq!(activities[0].id.as_str(), "i55610271");
    assert_eq!(activities[0].sport, Sport::Cycling);
    assert_eq!(activities[0].provider, ProviderId::new("intervals"));
}

#[tokio::test]
async fn lists_activities_without_query_params() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/athlete/0/activities"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    let activities = provider_for(&server)
        .activities(&Default::default())
        .await
        .expect("lists activities")
        .try_collect::<Vec<_>>()
        .await
        .expect("stream succeeds");

    assert!(activities.is_empty());
}

#[tokio::test]
async fn fetches_single_activity() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/activity/i55610271"))
        .respond_with(ResponseTemplate::new(200).set_body_json(activity_value()))
        .mount(&server)
        .await;

    let activity = provider_for(&server)
        .activity(&RemoteId::new("i55610271"))
        .await
        .expect("fetches activity");

    assert_eq!(activity.id.as_str(), "i55610271");
    assert_eq!(activity.sport, Sport::Cycling);
}

#[tokio::test]
async fn missing_activity_is_classified_as_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/activity/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("no such activity"))
        .mount(&server)
        .await;

    let error = provider_for(&server)
        .activity(&RemoteId::new("999"))
        .await
        .expect_err("fails");

    assert!(matches!(error, Error::NotFound(_)));
}

#[tokio::test]
async fn invalid_api_key_is_classified_as_unauthorized() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/athlete/0"))
        .respond_with(ResponseTemplate::new(401).set_body_string("invalid key"))
        .mount(&server)
        .await;

    let error = provider_for(&server).athlete().await.expect_err("fails");

    assert!(matches!(error, Error::Authentication(_)));
}

#[tokio::test]
async fn uploads_activity_file() {
    let file_path = std::env::temp_dir().join(format!(
        "oxisport-intervals-test-upload-{}",
        std::process::id()
    ));
    tokio::fs::write(&file_path, b"fit-bytes")
        .await
        .expect("writes temp file");

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/athlete/0/activities"))
        .and(query_param("name", "Morning ride"))
        .and(query_param("external_id", "ext-1"))
        .and(wiremock::matchers::body_string_contains("fit-bytes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(activity_value()))
        .mount(&server)
        .await;

    let activity = provider_for(&server)
        .upload_activity(&file_path, Some("Morning ride"), None, Some("ext-1"))
        .await
        .expect("upload succeeds");

    assert_eq!(activity.id.as_str(), "i55610271");
    assert_eq!(activity.sport, Sport::Cycling);

    let _ = tokio::fs::remove_file(&file_path).await;
}

#[tokio::test]
async fn upload_missing_file_fails_with_invalid_request() {
    let server = MockServer::start().await;

    let error = provider_for(&server)
        .upload_activity(
            std::path::Path::new("/nonexistent/activity.fit"),
            None,
            None,
            None,
        )
        .await
        .expect_err("fails");

    assert!(matches!(error, Error::InvalidRequest(_)));
}
