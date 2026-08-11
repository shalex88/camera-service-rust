use async_trait::async_trait;

use crate::core::DomainError;

/// A device port for image-stabilization control.
#[async_trait]
pub trait StabilizationCapability: Send + Sync {
    /// Enables or disables stabilization.
    async fn set_stabilization(&self, enabled: bool) -> Result<(), DomainError>;
    /// Reports whether stabilization is enabled.
    async fn stabilization_enabled(&self) -> Result<bool, DomainError>;
}
