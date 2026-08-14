//! Normalized domain entities.
//!
//! These are deliberately compact. Data that only one provider exposes
//! belongs in that provider's models, not here.

use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::model::{Cadence, Distance, GearKind, HeartRate, Power, ProviderId, RemoteId, Sport};

/// A recorded activity (run, ride, swim, ...).
///
/// Only summary-level data is normalized; detailed records (laps, splits,
/// streams, files) remain provider-specific for now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    /// Identifier inside the provider's system.
    pub id: RemoteId,
    /// The provider this activity came from.
    pub provider: ProviderId,
    /// Sport type.
    pub sport: Sport,
    /// Human-readable name, when the provider has one.
    pub name: Option<String>,
    /// Start time, when the provider exposes one.
    pub start_time: Option<DateTime<Utc>>,
    /// Total distance, when applicable.
    pub distance: Option<Distance>,
    /// Total duration, when exposed.
    pub duration: Option<Duration>,
    /// Average heart rate, when available.
    pub heart_rate: Option<HeartRate>,
    /// Average power, when available.
    pub power: Option<Power>,
    /// Average cadence, when available.
    pub cadence: Option<Cadence>,
}

/// A route (a saved track or planned course).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    /// Identifier inside the provider's system.
    pub id: RemoteId,
    /// The provider this route came from.
    pub provider: ProviderId,
    /// Human-readable name.
    pub name: Option<String>,
    /// Route length, when exposed.
    pub distance: Option<Distance>,
}

/// A structured workout (e.g. an interval session definition).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workout {
    /// Identifier inside the provider's system.
    pub id: RemoteId,
    /// The provider this workout came from.
    pub provider: ProviderId,
    /// Human-readable name.
    pub name: String,
    /// Sport type, when the provider distinguishes.
    pub sport: Option<Sport>,
    /// Planned duration, when exposed.
    pub duration: Option<Duration>,
}

/// An athlete (the account owner on the provider).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Athlete {
    /// Identifier inside the provider's system.
    pub id: RemoteId,
    /// The provider this athlete belongs to.
    pub provider: ProviderId,
    /// Display name.
    pub name: Option<String>,
}

/// A piece of gear tracked by the provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gear {
    /// Identifier inside the provider's system.
    pub id: RemoteId,
    /// The provider this gear belongs to.
    pub provider: ProviderId,
    /// Human-readable name.
    pub name: String,
    /// Kind of gear, when known.
    pub kind: Option<GearKind>,
    /// Total distance accumulated on this gear, when exposed.
    pub distance: Option<Distance>,
}

/// A device registered with the provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    /// Identifier inside the provider's system.
    pub id: RemoteId,
    /// The provider this device belongs to.
    pub provider: ProviderId,
    /// Human-readable name.
    pub name: String,
    /// Device model, when exposed.
    pub model: Option<String>,
}
