use async_trait::async_trait;

use crate::entity::Device;
use crate::error::Result;
use crate::model::RemoteId;

/// Provides devices registered with a provider.
#[async_trait]
pub trait DeviceSource: Send + Sync {
    /// Lists all accessible devices.
    async fn devices(&self) -> Result<Vec<Device>>;
}

/// A course to upload to a device.
///
/// The initial shape is deliberately minimal; file bodies for courses will
/// be introduced once the streaming body types settle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourseUpload {
    /// Human-readable course name.
    pub name: String,
}

/// Sends courses to devices through a provider.
#[async_trait]
pub trait DeviceCourseSink: Send + Sync {
    /// Uploads a course to the device and returns the remote course id.
    async fn upload_course(&self, device: &RemoteId, course: CourseUpload) -> Result<RemoteId>;
}
