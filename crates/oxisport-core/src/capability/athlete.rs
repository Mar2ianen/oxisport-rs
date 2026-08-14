use async_trait::async_trait;

use crate::entity::Athlete;
use crate::error::Result;

/// Provides the connected athlete's profile from a provider.
#[async_trait]
pub trait AthleteSource: Send + Sync {
    /// Fetches the athlete profile.
    async fn athlete(&self) -> Result<Athlete>;
}
