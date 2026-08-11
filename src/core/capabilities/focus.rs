use async_trait::async_trait;

use crate::core::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Focus(u8);

impl Focus {
    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(100);

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

    pub const fn value(self) -> u8 {
        self.0
    }
}

#[async_trait]
pub trait FocusCapability: Send + Sync {
    async fn set_focus(&self, focus: Focus) -> Result<(), DomainError>;
    async fn focus(&self) -> Result<Focus, DomainError>;
}
