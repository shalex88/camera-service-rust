use async_trait::async_trait;

use crate::core::DomainError;

/// A normalized focus value in the inclusive range `0..=100`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Focus(u8);

impl Focus {
    /// The minimum normalized focus value.
    pub const MIN: Self = Self(0);
    /// The maximum normalized focus value.
    pub const MAX: Self = Self(100);

    /// Validates and constructs a normalized focus value.
    pub fn new(value: i64) -> Result<Self, DomainError> {
        let value = u8::try_from(value).map_err(|_| DomainError::InvalidArgument {
            field: "focus",
            value: value.to_string(),
            expected: "0..=100",
        })?;
        if value > Self::MAX.0 {
            return Err(DomainError::InvalidArgument {
                field: "focus",
                value: value.to_string(),
                expected: "0..=100",
            });
        }
        Ok(Self(value))
    }

    /// Returns the normalized value.
    pub const fn value(self) -> u8 {
        self.0
    }
}

/// A device port for normalized focus control.
#[async_trait]
pub trait FocusCapability: Send + Sync {
    /// Changes the device focus.
    async fn set_focus(&self, focus: Focus) -> Result<(), DomainError>;
    /// Reads the current device focus.
    async fn focus(&self) -> Result<Focus, DomainError>;
}
