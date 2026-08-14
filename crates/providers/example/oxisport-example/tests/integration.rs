//! End-to-end tests for the example provider against a wiremock server.

use futures_util::TryStreamExt;
use oxisport_core::{ActivitySource, Distance, Error, RemoteId, Sport};
use oxisport_example::{ExampleConfig, ExampleProvider};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const ACTIVITY_JSON: &str = r#"{
    "id": "42",
    "name": "Morning run",
    "sport": "running",
    "start_time": "2026-08-14T06:30:00Z",
    "distance_meters": 10000,
    "duration_seconds": 2700,
    "average_heart_rate": 152,
    "average_power": null,
    "average_cadence": 88
}"#;

const ACTIVITIES_JSON: &str = r#"[
    {
        "id": "42",
        "name": "Morning run",
        "sport": "running",
        "start_time": "2026-08-14T06:30:00Z",
        "distance_meters": 10000,
        "duration_seconds": 2700,
        "average_heart_rate": 152
    },
    {
        "id": "43",
        "sport": "cycling",
        "start_time": "2026-08-13T17:00:00Z",
        "distance_meters": 40200,
        "duration_seconds": 5400,
        "average_power": 210
    }
]"#;

fn provider_for(server: &MockServer) -> ExampleProvider {
    ExampleProvider::new(ExampleConfig {
        base_url: server.uri(),
        user_agent: None,
    })
    .expect("provider builds against mock server")
}

#[tokio::test]
async fn activity_flows_from_wire_to_core_model() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/activities/42"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(ACTIVITY_JSON, "application/json"))
        .mount(&server)
        .await;

    let provider = provider_for(&server);
    let activity = provider
        .activity(&RemoteId::new("42"))
        .await
        .expect("activity");

    assert_eq!(activity.id.as_str(), "42");
    assert_eq!(activity.provider.as_str(), "example");
    assert_eq!(activity.sport, Sport::Running);
    assert_eq!(activity.distance, Some(Distance::from_meters(10_000)));
    assert_eq!(
        activity.duration,
        Some(std::time::Duration::from_secs(2700))
    );
    assert_eq!(
        activity.heart_rate,
        Some(oxisport_core::HeartRate::new(152))
    );
}

#[tokio::test]
async fn activities_streams_normalized_activities() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/activities"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(ACTIVITIES_JSON, "application/json"))
        .mount(&server)
        .await;

    let provider = provider_for(&server);
    let stream = provider
        .activities(&Default::default())
        .await
        .expect("stream");
    let activities: Vec<_> = stream.try_collect().await.expect("stream succeeds");

    assert_eq!(activities.len(), 2);
    assert_eq!(activities[0].id.as_str(), "42");
    assert_eq!(activities[0].sport, Sport::Running);
    assert_eq!(activities[1].id.as_str(), "43");
    assert_eq!(activities[1].sport, Sport::Cycling);
    assert_eq!(activities[1].power, Some(oxisport_core::Power::new(210)));
    assert_eq!(activities[1].heart_rate, None);
}

#[tokio::test]
async fn missing_activity_is_classified_as_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/activities/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("no such activity"))
        .mount(&server)
        .await;

    let provider = provider_for(&server);
    let error = provider
        .activity(&RemoteId::new("999"))
        .await
        .expect_err("fails");

    assert!(matches!(error, Error::NotFound(_)));
}

#[tokio::test]
async fn rate_limited_is_propagated() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/activities"))
        .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "5"))
        .mount(&server)
        .await;

    let provider = provider_for(&server);
    let error = match provider.activities(&Default::default()).await {
        Err(error) => error,
        Ok(_) => panic!("expected an error"),
    };

    match error {
        Error::RateLimited { retry_after } => {
            assert_eq!(retry_after, Some(std::time::Duration::from_secs(5)));
        }
        other => panic!("expected RateLimited, got {other:?}"),
    }
}
