//! OxiSport example (mock) provider adapter.
//!
//! This provider proves the framework architecture end to end:
//!
//! ```text
//! mock wire API -> generated raw client (oxisport-example-raw)
//!                -> adapter (this crate)
//!                -> oxisport_core::Activity
//! ```
//!
//! It is a demo, not a real service: the default base URL is
//! `https://example.invalid` and every remote call fails unless the client
//! is pointed at a mock server (as the integration tests do).

pub mod mapping;
pub mod provider;

pub use provider::{ExampleConfig, ExampleProvider};
