use async_trait::async_trait;

use crate::core::DomainError;

/// A device port for autofocus control.
#[async_trait]
pub trait AutoFocusCapability: Send + Sync {
    /// Enables or disables autofocus.
    async fn set_auto_focus(&self, enabled: bool) -> Result<(), DomainError>;
    /// Reports whether autofocus is enabled.
    async fn auto_focus_enabled(&self) -> Result<bool, DomainError>;
}
