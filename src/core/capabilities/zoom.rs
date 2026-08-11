use async_trait::async_trait;

use crate::core::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zoom(u8);

impl Zoom {
    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(100);

    pub fn new(value: i64) -> Result<Self, DomainError> {
        let value = u8::try_from(value).map_err(|_| DomainError::InvalidArgument {
            field: "zoom",
            value: value.to_string(),
            expected: "0..=100",
        })?;
        if value > Self::MAX.0 {
            return Err(DomainError::InvalidArgument {
                field: "zoom",
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
pub trait ZoomCapability: Send + Sync {
    async fn set_zoom(&self, zoom: Zoom) -> Result<(), DomainError>;
    async fn zoom(&self) -> Result<Zoom, DomainError>;
}
