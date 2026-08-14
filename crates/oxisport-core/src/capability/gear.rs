use async_trait::async_trait;

use crate::entity::Gear;
use crate::error::Result;

/// Provides gear tracked by a provider.
#[async_trait]
pub trait GearSource: Send + Sync {
    /// Lists all accessible gear.
    async fn gears(&self) -> Result<Vec<Gear>>;
}
