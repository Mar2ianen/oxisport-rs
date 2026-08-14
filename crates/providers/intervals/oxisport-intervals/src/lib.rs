//! Intervals.icu provider adapter.
//!
//! Maps the Intervals.icu wire API into the normalized `oxisport-core`
//! model:
//!
//! ```text
//! Intervals.icu API -> raw wire client (oxisport-intervals-raw)
//!                   -> adapter (this crate)
//!                   -> oxisport_core::{Activity, Athlete}
//! ```
//!
//! Authentication uses the personal API key from
//! <https://intervals.icu/settings>, sent as the `X-API-KEY` header.
//!
//! Implemented capabilities: [`AthleteSource`] and [`ActivitySource`].
//! Activity-file upload is provider-specific and exposed directly on
//! [`IntervalsProvider::upload_activity`].

pub mod mapping;
pub mod provider;

pub use provider::{DEFAULT_BASE_URL, IntervalsConfig, IntervalsProvider};
