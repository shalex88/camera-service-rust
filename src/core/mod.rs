pub mod capabilities;
pub mod device;
pub mod error;

pub use capabilities::{
    AutoFocusCapability, Capability, DeviceInfo, Focus, FocusCapability, InfoCapability,
    StabilizationCapability, Zoom, ZoomCapability,
};
pub use device::{DeviceLifecycle, DevicePorts, DevicePortsBuilder};
pub use error::DomainError;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::{
        Capability, DeviceInfo, DeviceLifecycle, DevicePorts, DomainError, Focus, FocusCapability,
        InfoCapability, Zoom, ZoomCapability,
    };

    struct TestDevice;

    #[async_trait]
    impl DeviceLifecycle for TestDevice {
        async fn open(&self) -> Result<(), DomainError> {
            Ok(())
        }

        async fn close(&self) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[async_trait]
    impl ZoomCapability for TestDevice {
        async fn set_zoom(&self, _zoom: Zoom) -> Result<(), DomainError> {
            Ok(())
        }

        async fn zoom(&self) -> Result<Zoom, DomainError> {
            Ok(Zoom::MIN)
        }
    }

    #[async_trait]
    impl FocusCapability for TestDevice {
        async fn set_focus(&self, _focus: Focus) -> Result<(), DomainError> {
            Ok(())
        }

        async fn focus(&self) -> Result<Focus, DomainError> {
            Ok(Focus::MIN)
        }
    }

    #[async_trait]
    impl InfoCapability for TestDevice {
        async fn info(&self) -> Result<DeviceInfo, DomainError> {
            Ok(DeviceInfo::new("test camera"))
        }
    }

    #[test]
    fn zoom_accepts_only_normalized_boundaries() {
        assert_eq!(Zoom::new(0).expect("zero is valid").value(), 0);
        assert_eq!(Zoom::new(100).expect("one hundred is valid").value(), 100);
        assert!(matches!(
            Zoom::new(-1),
            Err(DomainError::InvalidArgument { .. })
        ));
        assert!(matches!(
            Zoom::new(101),
            Err(DomainError::InvalidArgument { .. })
        ));
    }

    #[test]
    fn focus_accepts_only_normalized_boundaries() {
        assert_eq!(Focus::new(0).expect("zero is valid").value(), 0);
        assert_eq!(Focus::new(100).expect("one hundred is valid").value(), 100);
        assert!(matches!(
            Focus::new(-1),
            Err(DomainError::InvalidArgument { .. })
        ));
        assert!(matches!(
            Focus::new(101),
            Err(DomainError::InvalidArgument { .. })
        ));
    }

    #[test]
    fn device_ports_derive_capabilities_from_registered_ports() {
        let device = Arc::new(TestDevice);
        let ports = DevicePorts::builder(device.clone())
            .with_zoom(device.clone())
            .with_focus(device.clone())
            .with_info(device)
            .build();

        assert_eq!(
            ports.capabilities(),
            vec![Capability::Zoom, Capability::Focus, Capability::Info]
        );
        assert!(ports.autofocus().is_none());
        assert!(ports.stabilization().is_none());
    }
}
