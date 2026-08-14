use std::pin::Pin;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_core::Stream;

use crate::entity::Activity;
use crate::error::Result;
use crate::model::{Distance, RemoteId, Sport};

/// A streaming collection of normalized [`Activity`] values.
///
/// Designed so providers can lazily page through a remote API instead of
/// buffering every page into a `Vec`.
pub type ActivityStream<'a> = Pin<Box<dyn Stream<Item = Result<Activity>> + Send + 'a>>;

/// Filters for listing activities.
#[derive(Debug, Clone, Default)]
pub struct ActivityQuery {
    /// Maximum number of activities to return.
    pub limit: Option<u32>,
    /// Only activities that started after this time.
    pub after: Option<DateTime<Utc>>,
    /// Only activities that started before this time.
    pub before: Option<DateTime<Utc>>,
}

/// Provides activities from a provider.
///
/// The initial shape of this trait is a sketch and will be refined while
/// real providers are implemented.
#[async_trait]
pub trait ActivitySource: Send + Sync {
    /// Fetches a single activity.
    async fn activity(&self, id: &RemoteId) -> Result<Activity>;

    /// Streams activities matching the query.
    async fn activities(&self, query: &ActivityQuery) -> Result<ActivityStream<'_>>;
}

/// Input data for creating an activity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityInput {
    /// Sport type.
    pub sport: Sport,
    /// Human-readable name.
    pub name: Option<String>,
    /// Start time.
    pub start_time: Option<DateTime<Utc>>,
    /// Total distance.
    pub distance: Option<Distance>,
    /// Total duration.
    pub duration: Option<Duration>,
}

/// Accepts activities into a provider.
#[async_trait]
pub trait ActivitySink: Send + Sync {
    /// Creates an activity and returns the stored representation.
    async fn create_activity(&self, input: ActivityInput) -> Result<Activity>;
}
