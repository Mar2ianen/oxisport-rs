//! Strava provider adapter (scaffold).
//!
//! Status: scaffold only. Not implemented in milestone 0.
//!
//! Planned responsibilities:
//! - OAuth 2.0 authentication and token refresh;
//! - `ActivitySource` / `RouteSource` and other capabilities Strava
//!   actually supports, mapped into `oxisport-core`;
//! - conversion between `oxisport-strava-raw` wire models and the
//!   normalized model;
//! - access to the raw client for Strava-specific operations.
//!
//! Planned dependencies once implemented: `oxisport-strava-raw`,
//! `oxisport-core`, `oxisport-runtime`.
//!
//! API source: Strava API v3 — https://developers.strava.com/docs/reference/
