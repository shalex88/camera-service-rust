use async_trait::async_trait;

use crate::core::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo(String);

impl DeviceInfo {
    pub fn new(info: impl Into<String>) -> Self {
        Self(info.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[async_trait]
pub trait InfoCapability: Send + Sync {
    async fn info(&self) -> Result<DeviceInfo, DomainError>;
}
