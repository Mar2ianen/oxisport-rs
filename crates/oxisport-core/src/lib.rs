//! OxiSport normalized domain model and capability traits.
//!
//! `oxisport-core` represents the meaningful intersection of concepts across
//! sport and fitness services. It must never depend on a provider crate, and
//! provider wire models must never leak into it.
//!
//! Shared abstractions are capability-based: providers implement only the
//! capabilities they actually support, and provider-specific functionality
//! stays reachable through provider and raw client APIs.

pub mod capability;
pub mod entity;
pub mod error;
pub mod model;

pub use capability::{
    ActivityInput, ActivityQuery, ActivitySink, ActivitySource, ActivityStream, AthleteSource,
    CourseUpload, DeviceCourseSink, DeviceSource, GearSource, RouteInput, RouteQuery, RouteSink,
    RouteSource, RouteStream, WorkoutInput, WorkoutSink, WorkoutSource,
};
pub use entity::{Activity, Athlete, Device, Gear, Route, Workout};
pub use error::{Error, RemoteError, Result};
pub use model::{Cadence, Distance, GearKind, HeartRate, Power, ProviderId, RemoteId, Sport};
