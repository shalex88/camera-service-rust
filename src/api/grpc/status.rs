use tonic::Status;

use crate::core::DomainError;

impl From<DomainError> for Status {
    fn from(error: DomainError) -> Self {
        let message = error.to_string();
        match error {
            DomainError::InvalidArgument { .. } => Self::invalid_argument(message),
            DomainError::UnsupportedCapability(_) => Self::unimplemented(message),
            DomainError::NotRunning => Self::failed_precondition(message),
            DomainError::DeviceUnavailable(_) => Self::unavailable(message),
            DomainError::Device(_) => Self::internal(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use tonic::{Code, Status};

    use crate::core::DomainError;

    #[test]
    fn maps_domain_errors_to_stable_grpc_codes() {
        let invalid = DomainError::InvalidArgument {
            field: "zoom",
            value: "101".to_owned(),
            expected: "0..=100",
        };

        assert_eq!(Status::from(invalid).code(), Code::InvalidArgument);
        assert_eq!(
            Status::from(DomainError::UnsupportedCapability("autofocus")).code(),
            Code::Unimplemented
        );
        assert_eq!(
            Status::from(DomainError::NotRunning).code(),
            Code::FailedPrecondition
        );
        assert_eq!(
            Status::from(DomainError::DeviceUnavailable("closed".to_owned())).code(),
            Code::Unavailable
        );
        assert_eq!(
            Status::from(DomainError::Device("failed".to_owned())).code(),
            Code::Internal
        );
    }
}
