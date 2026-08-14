//! Capability-based provider traits.
//!
//! There is intentionally no giant `Provider` interface. A provider
//! implements only the capabilities it actually supports, and provider
//! A never has to pretend to be provider B.
//!
//! `Source` traits read data; `Sink` traits write data. Pagination-heavy
//! sources are designed around async [`Stream`]s rather than eager `Vec`
//! collection.

mod activity;
mod athlete;
mod device;
mod gear;
mod route;
mod workout;

pub use activity::{ActivityInput, ActivityQuery, ActivitySink, ActivitySource, ActivityStream};
pub use athlete::AthleteSource;
pub use device::{CourseUpload, DeviceCourseSink, DeviceSource};
pub use gear::GearSource;
pub use route::{RouteInput, RouteQuery, RouteSink, RouteSource, RouteStream};
pub use workout::{WorkoutInput, WorkoutSink, WorkoutSource};
