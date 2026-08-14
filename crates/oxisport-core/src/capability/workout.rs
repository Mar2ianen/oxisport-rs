use std::time::Duration;

use async_trait::async_trait;

use crate::entity::Workout;
use crate::error::Result;
use crate::model::Sport;

/// Provides workouts from a provider.
#[async_trait]
pub trait WorkoutSource: Send + Sync {
    /// Lists all accessible workouts.
    async fn workouts(&self) -> Result<Vec<Workout>>;
}

/// Input data for creating a workout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkoutInput {
    /// Human-readable name.
    pub name: String,
    /// Sport type.
    pub sport: Sport,
    /// Planned duration.
    pub duration: Option<Duration>,
}

/// Accepts workouts into a provider.
#[async_trait]
pub trait WorkoutSink: Send + Sync {
    /// Creates a workout and returns the stored representation.
    async fn create_workout(&self, input: WorkoutInput) -> Result<Workout>;
}
