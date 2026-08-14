//! Identifiers and measurement support types.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Identifies a provider (for example `strava` or `garmin`).
///
/// Providers use lowercase stable identifiers, e.g. `strava`, `intervals`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProviderId(String);

impl ProviderId {
    /// Creates a provider identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ProviderId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An identifier of a resource inside a provider's system.
///
/// Remote identifiers are opaque strings; only the provider knows their
/// internal meaning.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RemoteId(String);

impl RemoteId {
    /// Creates a remote identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RemoteId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for RemoteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A sport or activity type.
///
/// Known sports map to stable variants; anything else is preserved in
/// [`Sport::Other`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Sport {
    /// Running.
    Running,
    /// Road, mountain or indoor cycling.
    Cycling,
    /// Swimming (pool or open water).
    Swimming,
    /// Triathlon (multi-sport).
    Triathlon,
    /// Hiking.
    Hiking,
    /// Walking.
    Walking,
    /// Skiing.
    Skiing,
    /// Rowing.
    Rowing,
    /// Any sport not covered by the known variants.
    Other(String),
}

impl Sport {
    /// Returns the stable lowercase name of the sport.
    pub fn as_str(&self) -> &str {
        match self {
            Sport::Running => "running",
            Sport::Cycling => "cycling",
            Sport::Swimming => "swimming",
            Sport::Triathlon => "triathlon",
            Sport::Hiking => "hiking",
            Sport::Walking => "walking",
            Sport::Skiing => "skiing",
            Sport::Rowing => "rowing",
            Sport::Other(name) => name,
        }
    }
}

impl From<&str> for Sport {
    fn from(value: &str) -> Self {
        match value {
            "running" => Sport::Running,
            "cycling" => Sport::Cycling,
            "swimming" => Sport::Swimming,
            "triathlon" => Sport::Triathlon,
            "hiking" => Sport::Hiking,
            "walking" => Sport::Walking,
            "skiing" => Sport::Skiing,
            "rowing" => Sport::Rowing,
            other => Sport::Other(other.to_string()),
        }
    }
}

impl fmt::Display for Sport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A distance.
///
/// Internally stored in whole meters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Distance {
    meters: u64,
}

impl Distance {
    /// Creates a distance from whole meters.
    pub const fn from_meters(meters: u64) -> Self {
        Self { meters }
    }

    /// Creates a distance from kilometers.
    pub fn from_kilometers(kilometers: f64) -> Self {
        Self {
            meters: (kilometers * 1000.0).round() as u64,
        }
    }

    /// Returns the distance in whole meters.
    pub const fn meters(self) -> u64 {
        self.meters
    }

    /// Returns the distance in kilometers.
    pub fn kilometers(self) -> f64 {
        self.meters as f64 / 1000.0
    }
}

impl From<u64> for Distance {
    fn from(meters: u64) -> Self {
        Self::from_meters(meters)
    }
}

impl fmt::Display for Distance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} m", self.meters)
    }
}

/// A heart rate value in beats per minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HeartRate(u16);

impl HeartRate {
    /// Creates a heart rate from beats per minute.
    pub const fn new(bpm: u16) -> Self {
        Self(bpm)
    }

    /// Returns the value in beats per minute.
    pub const fn bpm(self) -> u16 {
        self.0
    }
}

/// A power value in watts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Power(u16);

impl Power {
    /// Creates a power value from watts.
    pub const fn new(watts: u16) -> Self {
        Self(watts)
    }

    /// Returns the value in watts.
    pub const fn watts(self) -> u16 {
        self.0
    }
}

/// A cadence value in revolutions per minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cadence(u16);

impl Cadence {
    /// Creates a cadence value from revolutions per minute.
    pub const fn new(rpm: u16) -> Self {
        Self(rpm)
    }

    /// Returns the value in revolutions per minute.
    pub const fn rpm(self) -> u16 {
        self.0
    }
}

/// The kind of a piece of gear.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GearKind {
    /// A bicycle.
    Bike,
    /// Running shoes.
    Shoes,
    /// Anything else.
    Other,
}

#[cfg(test)]
mod tests {
    use super::{Cadence, Distance, HeartRate, Power, ProviderId, RemoteId, Sport};

    #[test]
    fn sport_round_trips_known_names() {
        for sport in [
            Sport::Running,
            Sport::Cycling,
            Sport::Swimming,
            Sport::Triathlon,
            Sport::Hiking,
            Sport::Walking,
            Sport::Skiing,
            Sport::Rowing,
        ] {
            assert_eq!(Sport::from(sport.as_str()), sport);
        }
    }

    #[test]
    fn unknown_sport_is_preserved() {
        assert_eq!(
            Sport::from("inline-skating"),
            Sport::Other("inline-skating".to_string())
        );
    }

    #[test]
    fn distance_conversions() {
        let d = Distance::from_kilometers(10.0);
        assert_eq!(d.meters(), 10_000);
        assert_eq!(d.kilometers(), 10.0);
        assert_eq!(Distance::from_meters(5).kilometers(), 0.005);
    }

    #[test]
    fn newtype_accessors() {
        assert_eq!(HeartRate::new(150).bpm(), 150);
        assert_eq!(Power::new(240).watts(), 240);
        assert_eq!(Cadence::new(88).rpm(), 88);
    }

    #[test]
    fn identifiers_expose_str() {
        assert_eq!(RemoteId::new("42").as_str(), "42");
        assert_eq!(ProviderId::new("strava").as_str(), "strava");
        assert_eq!(ProviderId::from("garmin").to_string(), "garmin");
    }
}
