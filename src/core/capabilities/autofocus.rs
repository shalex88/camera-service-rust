use async_trait::async_trait;

use crate::core::DomainError;

#[async_trait]
pub trait AutoFocusCapability: Send + Sync {
    async fn set_auto_focus(&self, enabled: bool) -> Result<(), DomainError>;
    async fn auto_focus_enabled(&self) -> Result<bool, DomainError>;
}
