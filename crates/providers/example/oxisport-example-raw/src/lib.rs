//! Raw wire client for the OxiSport example (mock) service.
//!
//! The `generated` module is produced by `oxisport-codegen` from
//! `specs/examples/example.yaml` and committed to git. Do not edit it by
//! hand; regenerate with `cargo xtask codegen`.
//!
//! This crate mirrors the wire API of the mock service only: it performs no
//! normalization and knows nothing about `oxisport-core` entities. Mapping
//! into the normalized model happens in the `oxisport-example` adapter.

pub mod generated;

pub use generated::*;
