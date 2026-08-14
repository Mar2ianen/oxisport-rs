//! Garmin Connect provider adapter (scaffold).
//!
//! Status: scaffold only. Not implemented in milestone 0.
//!
//! Garmin Connect does not offer an official public API. If this provider
//! is implemented, it will rely on reverse-engineered or unofficial
//! endpoints; that fact will be clearly documented in this crate.
//!
//! Planned responsibilities:
//! - session-based authentication (careful, undocumented);
//! - `ActivitySource` / `ActivitySink` with original FIT/GPX/TCX transfer;
//! - `DeviceSource` and `DeviceCourseSink`;
//! - conversion between `oxisport-garmin-raw` wire models and the
//!   normalized model.
//!
//! Planned dependencies once implemented: `oxisport-garmin-raw`,
//! `oxisport-core`, `oxisport-runtime`, `oxisport-files`.
