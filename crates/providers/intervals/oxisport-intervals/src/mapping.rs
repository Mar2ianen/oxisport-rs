//! Conversions between the raw wire model and `oxisport-core` entities.
//!
//! All conversions are pure functions so they can be unit tested with
//! representative payloads, without any network.

use std::time::Duration;

use chrono::{DateTime, NaiveDateTime, Utc};
use oxisport_core::{
    Activity, Athlete, Cadence, Distance, HeartRate, Power, ProviderId, RemoteId, Sport,
};
use oxisport_intervals_raw::{ActivitySummary, AthleteResponse};

/// Converts a raw activity summary into the normalized model.
pub fn activity(provider: &ProviderId, raw: ActivitySummary) -> Activity {
    Activity {
        id: RemoteId::new(raw.id),
        provider: provider.clone(),
        sport: sport(&raw.r#type),
        name: raw.name,
        start_time: parse_local_start(&raw.start_date_local),
        distance: raw
            .distance
            .map(|meters| Distance::from_meters(meters.round() as u64)),
        duration: raw
            .moving_time
            .or(raw.elapsed_time)
            .map(|secs| Duration::from_secs(u64::from(secs))),
        heart_rate: raw
            .average_heartrate
            .map(|bpm| HeartRate::new(bpm.round() as u16)),
        power: raw
            .average_watts
            .map(|watts| Power::new(watts.round() as u16)),
        cadence: raw
            .average_cadence
            .map(|rpm| Cadence::new(rpm.round() as u16)),
    }
}

/// Converts a raw athlete response into the normalized model.
pub fn athlete(provider: &ProviderId, raw: AthleteResponse) -> Athlete {
    let name = raw
        .display_name
        .or_else(|| match (raw.first_name, raw.last_name) {
            (Some(first), Some(last)) => Some(format!("{first} {last}")),
            (Some(first), None) => Some(first),
            (None, Some(last)) => Some(last),
            (None, None) => None,
        });
    Athlete {
        id: RemoteId::new(raw.id.to_string()),
        provider: provider.clone(),
        name,
    }
}

/// Maps an Intervals.icu activity type to the normalized sport.
pub fn sport(value: &str) -> Sport {
    match value.to_ascii_lowercase().as_str() {
        "run" | "trailrun" => Sport::Running,
        "ride" | "virtualride" | "ride indoor" | "mountainbikeride" | "gravelride" => {
            Sport::Cycling
        }
        "swim" => Sport::Swimming,
        "triathlon" => Sport::Triathlon,
        "hike" => Sport::Hiking,
        "walk" => Sport::Walking,
        "ski" => Sport::Skiing,
        "row" => Sport::Rowing,
        other => Sport::Other(other.to_string()),
    }
}

/// Intervals.icu reports local start times without a timezone; the athlete
/// timezone is unknown to the API client, so the value is treated as UTC.
fn parse_local_start(value: &str) -> Option<DateTime<Utc>> {
    let naive = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M"))
        .ok()?;
    Some(DateTime::from_naive_utc_and_offset(naive, Utc))
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use oxisport_core::{Distance, ProviderId};
    use oxisport_intervals_raw::{ActivitySummary, AthleteResponse};

    use super::{activity, athlete, parse_local_start, sport};

    fn summary() -> ActivitySummary {
        serde_json::from_str(
            r#"{
                "id": "i55610271",
                "name": "Morning ride",
                "type": "Ride",
                "start_date_local": "2026-08-14T06:30:00",
                "distance": 45000.0,
                "moving_time": 5400,
                "elapsed_time": 5670,
                "average_heartrate": 139,
                "average_watts": 212.5,
                "average_cadence": 88
            }"#,
        )
        .expect("parses fixture")
    }

    #[test]
    fn maps_full_activity_payload() {
        let provider = ProviderId::new("intervals");
        let mapped = activity(&provider, summary());

        assert_eq!(mapped.id.as_str(), "i55610271");
        assert_eq!(mapped.provider, provider);
        assert_eq!(mapped.sport, oxisport_core::Sport::Cycling);
        assert_eq!(mapped.name.as_deref(), Some("Morning ride"));
        assert_eq!(
            mapped.start_time,
            Some(Utc.with_ymd_and_hms(2026, 8, 14, 6, 30, 0).unwrap())
        );
        assert_eq!(mapped.distance, Some(Distance::from_meters(45_000)));
        assert_eq!(mapped.duration, Some(std::time::Duration::from_secs(5400)));
        assert_eq!(mapped.heart_rate, Some(oxisport_core::HeartRate::new(139)));
        assert_eq!(mapped.power, Some(oxisport_core::Power::new(213)));
        assert_eq!(mapped.cadence, Some(oxisport_core::Cadence::new(88)));
    }

    #[test]
    fn maps_minimal_activity_payload() {
        let raw: ActivitySummary = serde_json::from_str(
            r#"{
                "id": "7",
                "type": "Strength",
                "start_date_local": "2026-01-01T09:00"
            }"#,
        )
        .expect("parses fixture");

        let mapped = activity(&ProviderId::new("intervals"), raw);

        assert_eq!(mapped.name, None);
        assert_eq!(
            mapped.sport,
            oxisport_core::Sport::Other("strength".to_string())
        );
        assert_eq!(
            mapped.start_time,
            Some(Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap())
        );
        assert_eq!(mapped.distance, None);
        assert_eq!(mapped.duration, None);
        assert_eq!(mapped.heart_rate, None);
        assert_eq!(mapped.power, None);
        assert_eq!(mapped.cadence, None);
    }

    #[test]
    fn maps_known_and_unknown_sports() {
        assert_eq!(sport("Run"), oxisport_core::Sport::Running);
        assert_eq!(sport("TrailRun"), oxisport_core::Sport::Running);
        assert_eq!(sport("Ride Indoor"), oxisport_core::Sport::Cycling);
        assert_eq!(sport("VirtualRide"), oxisport_core::Sport::Cycling);
        assert_eq!(sport("Swim"), oxisport_core::Sport::Swimming);
        assert_eq!(sport("Triathlon"), oxisport_core::Sport::Triathlon);
        assert_eq!(sport("Hike"), oxisport_core::Sport::Hiking);
        assert_eq!(sport("Walk"), oxisport_core::Sport::Walking);
        assert_eq!(sport("Ski"), oxisport_core::Sport::Skiing);
        assert_eq!(sport("Row"), oxisport_core::Sport::Rowing);
        assert_eq!(
            sport("Crossfit"),
            oxisport_core::Sport::Other("crossfit".to_string())
        );
    }

    #[test]
    fn parses_local_start_times() {
        assert_eq!(
            parse_local_start("2026-08-14T06:30:00"),
            Some(Utc.with_ymd_and_hms(2026, 8, 14, 6, 30, 0).unwrap())
        );
        assert_eq!(
            parse_local_start("2026-08-14T06:30"),
            Some(Utc.with_ymd_and_hms(2026, 8, 14, 6, 30, 0).unwrap())
        );
        assert_eq!(parse_local_start("not-a-date"), None);
    }

    #[test]
    fn maps_athlete_with_display_name() {
        let raw: AthleteResponse = serde_json::from_str(
            r#"{
                "id": 2049151,
                "first_name": "John",
                "last_name": "Doe",
                "display_name": "jdoe"
            }"#,
        )
        .expect("parses fixture");

        let mapped = athlete(&ProviderId::new("intervals"), raw);

        assert_eq!(mapped.id.as_str(), "2049151");
        assert_eq!(mapped.name.as_deref(), Some("jdoe"));
    }

    #[test]
    fn maps_athlete_without_display_name() {
        let raw: AthleteResponse = serde_json::from_str(
            r#"{
                "id": 7,
                "first_name": "John",
                "last_name": "Doe"
            }"#,
        )
        .expect("parses fixture");

        let mapped = athlete(&ProviderId::new("intervals"), raw);

        assert_eq!(mapped.name.as_deref(), Some("John Doe"));
    }

    #[test]
    fn maps_anonymous_athlete() {
        let raw: AthleteResponse = serde_json::from_str(r#"{"id": 7}"#).expect("parses fixture");

        let mapped = athlete(&ProviderId::new("intervals"), raw);

        assert_eq!(mapped.name, None);
    }
}
