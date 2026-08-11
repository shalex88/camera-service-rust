use async_trait::async_trait;

use crate::core::DomainError;

/// Human-readable device information returned by the core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo(String);

impl DeviceInfo {
    /// Wraps device information supplied by an adapter.
    pub fn new(info: impl Into<String>) -> Self {
        Self(info.into())
    }

    /// Borrows the device information text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A device port for human-readable device information.
#[async_trait]
pub trait InfoCapability: Send + Sync {
    /// Reads information from the device.
    async fn info(&self) -> Result<DeviceInfo, DomainError>;
}
