use async_trait::async_trait;

use crate::core::DomainError;

/// A normalized zoom value in the inclusive range `0..=100`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zoom(u8);

impl Zoom {
    /// The minimum normalized zoom value.
    pub const MIN: Self = Self(0);
    /// The maximum normalized zoom value.
    pub const MAX: Self = Self(100);

    /// Validates and constructs a normalized zoom value.
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

    /// Returns the normalized value.
    pub const fn value(self) -> u8 {
        self.0
    }
}

/// A device port for normalized zoom control.
#[async_trait]
pub trait ZoomCapability: Send + Sync {
    /// Changes the device zoom.
    async fn set_zoom(&self, zoom: Zoom) -> Result<(), DomainError>;
    /// Reads the current device zoom.
    async fn zoom(&self) -> Result<Zoom, DomainError>;
}
