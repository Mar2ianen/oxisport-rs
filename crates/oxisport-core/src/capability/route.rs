use std::pin::Pin;

use async_trait::async_trait;
use futures_core::Stream;

use crate::entity::Route;
use crate::error::Result;
use crate::model::{Distance, RemoteId, Sport};

/// A streaming collection of normalized [`Route`] values.
pub type RouteStream<'a> = Pin<Box<dyn Stream<Item = Result<Route>> + Send + 'a>>;

/// Filters for listing routes.
#[derive(Debug, Clone, Default)]
pub struct RouteQuery {
    /// Maximum number of routes to return.
    pub limit: Option<u32>,
}

/// Provides routes from a provider.
#[async_trait]
pub trait RouteSource: Send + Sync {
    /// Fetches a single route.
    async fn route(&self, id: &RemoteId) -> Result<Route>;

    /// Streams routes matching the query.
    async fn routes(&self, query: &RouteQuery) -> Result<RouteStream<'_>>;
}

/// Input data for creating a route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteInput {
    /// Human-readable name.
    pub name: String,
    /// Sport type, when known.
    pub sport: Option<Sport>,
    /// Route length.
    pub distance: Option<Distance>,
}

/// Accepts routes into a provider.
#[async_trait]
pub trait RouteSink: Send + Sync {
    /// Creates a route and returns the stored representation.
    async fn create_route(&self, input: RouteInput) -> Result<Route>;
}
