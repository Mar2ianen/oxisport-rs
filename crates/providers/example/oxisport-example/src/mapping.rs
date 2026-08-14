//! Conversions between the raw wire model and `oxisport-core` entities.
//!
//! All conversions are pure functions so they can be unit tested with
//! representative payloads, without any network.

use std::time::Duration;

use chrono::{DateTime, Utc};
use oxisport_core::{Activity, Cadence, Distance, HeartRate, Power, ProviderId, RemoteId, Sport};
use oxisport_example_raw::generated::ActivityResponse;

/// Converts a raw activity response into the normalized model.
pub fn activity(provider: &ProviderId, raw: ActivityResponse) -> Activity {
    Activity {
        id: RemoteId::new(raw.id),
        provider: provider.clone(),
        sport: sport(&raw.sport),
        name: raw.name,
        start_time: raw.start_time.parse::<DateTime<Utc>>().ok(),
        distance: Some(Distance::from_meters(raw.distance_meters)),
        duration: Some(Duration::from_secs(raw.duration_seconds)),
        heart_rate: raw.average_heart_rate.map(HeartRate::new),
        power: raw.average_power.map(Power::new),
        cadence: raw.average_cadence.map(Cadence::new),
    }
}

/// Maps a wire sport name to the normalized sport.
pub fn sport(value: &str) -> Sport {
    match value {
        "running" => Sport::Running,
        "cycling" => Sport::Cycling,
        "swimming" => Sport::Swimming,
        "triathlon" => Sport::Triathlon,
        "hiking" => Sport::Hiking,
        "walking" => Sport::Walking,
        other => Sport::Other(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use oxisport_core::{Distance, ProviderId};
    use oxisport_example_raw::generated::ActivityResponse;

    use super::{activity, sport};

    const ACTIVITY_JSON: &str = r#"
    {
        "id": "42",
        "name": "Morning run",
        "sport": "running",
        "start_time": "2026-08-14T06:30:00Z",
        "distance_meters": 10000,
        "duration_seconds": 2700,
        "average_heart_rate": 152,
        "average_power": null,
        "average_cadence": 88
    }
    "#;

    #[test]
    fn maps_full_activity_payload() {
        let raw: ActivityResponse = serde_json::from_str(ACTIVITY_JSON).expect("parses fixture");
        let provider = ProviderId::new("example");

        let mapped = activity(&provider, raw);

        assert_eq!(mapped.id.as_str(), "42");
        assert_eq!(mapped.provider, provider);
        assert_eq!(mapped.sport, oxisport_core::Sport::Running);
        assert_eq!(mapped.name.as_deref(), Some("Morning run"));
        assert_eq!(
            mapped.start_time,
            Some(Utc.with_ymd_and_hms(2026, 8, 14, 6, 30, 0).unwrap())
        );
        assert_eq!(mapped.distance, Some(Distance::from_meters(10_000)));
        assert_eq!(mapped.duration, Some(std::time::Duration::from_secs(2700)));
        assert_eq!(mapped.heart_rate, Some(oxisport_core::HeartRate::new(152)));
        assert_eq!(mapped.power, None);
        assert_eq!(mapped.cadence, Some(oxisport_core::Cadence::new(88)));
    }

    #[test]
    fn maps_minimal_activity_payload() {
        let raw: ActivityResponse = serde_json::from_str(
            r#"{
                "id": "7",
                "sport": "rowing",
                "start_time": "2026-01-01T00:00:00+02:00",
                "distance_meters": 5000,
                "duration_seconds": 1200
            }"#,
        )
        .expect("parses fixture");

        let mapped = activity(&ProviderId::new("example"), raw);

        assert_eq!(mapped.name, None);
        assert_eq!(
            mapped.sport,
            oxisport_core::Sport::Other("rowing".to_string())
        );
        assert_eq!(mapped.heart_rate, None);
        assert_eq!(
            mapped.start_time,
            Some(Utc.with_ymd_and_hms(2025, 12, 31, 22, 0, 0).unwrap())
        );
    }

    #[test]
    fn maps_known_and_unknown_sports() {
        assert_eq!(sport("running"), oxisport_core::Sport::Running);
        assert_eq!(sport("cycling"), oxisport_core::Sport::Cycling);
        assert_eq!(
            sport("inline-skating"),
            oxisport_core::Sport::Other("inline-skating".into())
        );
    }
}
