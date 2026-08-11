#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    #[error("invalid {field}: {value}; expected {expected}")]
    InvalidArgument {
        field: &'static str,
        value: String,
        expected: &'static str,
    },
    #[error("capability '{0}' is not supported")]
    UnsupportedCapability(&'static str),
    #[error("camera service is not running")]
    NotRunning,
    #[error("device is unavailable: {0}")]
    DeviceUnavailable(String),
    #[error("device operation failed: {0}")]
    Device(String),
}
