use async_trait::async_trait;

use crate::core::DomainError;

#[async_trait]
pub trait StabilizationCapability: Send + Sync {
    async fn set_stabilization(&self, enabled: bool) -> Result<(), DomainError>;
    async fn stabilization_enabled(&self) -> Result<bool, DomainError>;
}
