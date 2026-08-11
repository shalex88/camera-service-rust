use tokio::sync::{RwLock, RwLockReadGuard};

use crate::core::{Capability, DeviceInfo, DevicePorts, DomainError, Focus, Zoom};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LifecycleState {
    Created,
    Running,
    Stopped,
}

/// Coordinates device lifecycle and capability use cases for API adapters.
pub struct CameraCore {
    ports: DevicePorts,
    lifecycle: RwLock<LifecycleState>,
}

impl CameraCore {
    /// Creates a core around one device-port aggregate.
    pub fn new(ports: DevicePorts) -> Self {
        Self {
            ports,
            lifecycle: RwLock::new(LifecycleState::Created),
        }
    }

    /// Opens the device once and moves the core into its running state.
    pub async fn start(&self) -> Result<(), DomainError> {
        let mut lifecycle = self.lifecycle.write().await;
        match *lifecycle {
            LifecycleState::Created => {
                self.ports.lifecycle().open().await?;
                *lifecycle = LifecycleState::Running;
                Ok(())
            }
            LifecycleState::Running => Ok(()),
            LifecycleState::Stopped => Err(DomainError::NotRunning),
        }
    }

    /// Drains active operations, closes the device once, and stops the core.
    pub async fn stop(&self) -> Result<(), DomainError> {
        let mut lifecycle = self.lifecycle.write().await;
        match *lifecycle {
            LifecycleState::Created => {
                *lifecycle = LifecycleState::Stopped;
                Ok(())
            }
            LifecycleState::Running => {
                *lifecycle = LifecycleState::Stopped;
                self.ports.lifecycle().close().await
            }
            LifecycleState::Stopped => Ok(()),
        }
    }

    async fn require_running(&self) -> Result<RwLockReadGuard<'_, LifecycleState>, DomainError> {
        let lifecycle = self.lifecycle.read().await;
        if *lifecycle != LifecycleState::Running {
            return Err(DomainError::NotRunning);
        }
        Ok(lifecycle)
    }

    /// Changes zoom through the registered zoom port.
    pub async fn set_zoom(&self, zoom: Zoom) -> Result<(), DomainError> {
        let _lifecycle = self.require_running().await?;
        let capability = self
            .ports
            .zoom()
            .ok_or(DomainError::UnsupportedCapability("zoom"))?;
        capability.set_zoom(zoom).await
    }

    /// Reads zoom through the registered zoom port.
    pub async fn zoom(&self) -> Result<Zoom, DomainError> {
        let _lifecycle = self.require_running().await?;
        let capability = self
            .ports
            .zoom()
            .ok_or(DomainError::UnsupportedCapability("zoom"))?;
        capability.zoom().await
    }

    /// Changes zoom to its normalized minimum.
    pub async fn go_to_min_zoom(&self) -> Result<(), DomainError> {
        self.set_zoom(Zoom::MIN).await
    }

    /// Changes zoom to its normalized maximum.
    pub async fn go_to_max_zoom(&self) -> Result<(), DomainError> {
        self.set_zoom(Zoom::MAX).await
    }

    /// Changes focus through the registered focus port.
    pub async fn set_focus(&self, focus: Focus) -> Result<(), DomainError> {
        let _lifecycle = self.require_running().await?;
        let capability = self
            .ports
            .focus()
            .ok_or(DomainError::UnsupportedCapability("focus"))?;
        capability.set_focus(focus).await
    }

    /// Reads focus through the registered focus port.
    pub async fn focus(&self) -> Result<Focus, DomainError> {
        let _lifecycle = self.require_running().await?;
        let capability = self
            .ports
            .focus()
            .ok_or(DomainError::UnsupportedCapability("focus"))?;
        capability.focus().await
    }

    /// Reads information through the registered information port.
    pub async fn info(&self) -> Result<DeviceInfo, DomainError> {
        let _lifecycle = self.require_running().await?;
        let capability = self
            .ports
            .info()
            .ok_or(DomainError::UnsupportedCapability("info"))?;
        capability.info().await
    }

    /// Changes autofocus through the registered autofocus port.
    pub async fn set_auto_focus(&self, enabled: bool) -> Result<(), DomainError> {
        let _lifecycle = self.require_running().await?;
        let capability = self
            .ports
            .autofocus()
            .ok_or(DomainError::UnsupportedCapability("autofocus"))?;
        capability.set_auto_focus(enabled).await
    }

    /// Reads autofocus state through the registered autofocus port.
    pub async fn auto_focus_enabled(&self) -> Result<bool, DomainError> {
        let _lifecycle = self.require_running().await?;
        let capability = self
            .ports
            .autofocus()
            .ok_or(DomainError::UnsupportedCapability("autofocus"))?;
        capability.auto_focus_enabled().await
    }

    /// Changes stabilization through the registered stabilization port.
    pub async fn set_stabilization(&self, enabled: bool) -> Result<(), DomainError> {
        let _lifecycle = self.require_running().await?;
        let capability = self
            .ports
            .stabilization()
            .ok_or(DomainError::UnsupportedCapability("stabilization"))?;
        capability.set_stabilization(enabled).await
    }

    /// Reads stabilization state through the registered stabilization port.
    pub async fn stabilization_enabled(&self) -> Result<bool, DomainError> {
        let _lifecycle = self.require_running().await?;
        let capability = self
            .ports
            .stabilization()
            .ok_or(DomainError::UnsupportedCapability("stabilization"))?;
        capability.stabilization_enabled().await
    }

    /// Returns the capabilities derived from the registered ports.
    pub async fn capabilities(&self) -> Result<Vec<Capability>, DomainError> {
        let _lifecycle = self.require_running().await?;
        Ok(self.ports.capabilities())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use async_trait::async_trait;
    use tokio::sync::{Notify, RwLock};

    use super::CameraCore;
    use crate::core::{
        DeviceInfo, DeviceLifecycle, DevicePorts, DomainError, Focus, FocusCapability,
        InfoCapability, Zoom, ZoomCapability,
    };

    struct TestDevice {
        open_count: AtomicUsize,
        close_count: AtomicUsize,
        zoom: RwLock<Zoom>,
        focus: RwLock<Focus>,
        block_zoom: bool,
        zoom_started: Notify,
        zoom_release: Notify,
    }

    impl TestDevice {
        fn new(block_zoom: bool) -> Self {
            Self {
                open_count: AtomicUsize::new(0),
                close_count: AtomicUsize::new(0),
                zoom: RwLock::new(Zoom::MIN),
                focus: RwLock::new(Focus::MIN),
                block_zoom,
                zoom_started: Notify::new(),
                zoom_release: Notify::new(),
            }
        }
    }

    #[async_trait]
    impl DeviceLifecycle for TestDevice {
        async fn open(&self) -> Result<(), DomainError> {
            self.open_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn close(&self) -> Result<(), DomainError> {
            self.close_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[async_trait]
    impl ZoomCapability for TestDevice {
        async fn set_zoom(&self, zoom: Zoom) -> Result<(), DomainError> {
            if self.block_zoom {
                self.zoom_started.notify_one();
                self.zoom_release.notified().await;
            }
            *self.zoom.write().await = zoom;
            Ok(())
        }

        async fn zoom(&self) -> Result<Zoom, DomainError> {
            Ok(*self.zoom.read().await)
        }
    }

    #[async_trait]
    impl FocusCapability for TestDevice {
        async fn set_focus(&self, focus: Focus) -> Result<(), DomainError> {
            *self.focus.write().await = focus;
            Ok(())
        }

        async fn focus(&self) -> Result<Focus, DomainError> {
            Ok(*self.focus.read().await)
        }
    }

    #[async_trait]
    impl InfoCapability for TestDevice {
        async fn info(&self) -> Result<DeviceInfo, DomainError> {
            Ok(DeviceInfo::new("test camera"))
        }
    }

    fn service_with_supported_ports(block_zoom: bool) -> (Arc<CameraCore>, Arc<TestDevice>) {
        let device = Arc::new(TestDevice::new(block_zoom));
        let ports = DevicePorts::builder(device.clone())
            .with_zoom(device.clone())
            .with_focus(device.clone())
            .with_info(device.clone())
            .build();
        (Arc::new(CameraCore::new(ports)), device)
    }

    #[tokio::test]
    async fn rejects_operations_before_start() {
        let (core, _) = service_with_supported_ports(false);

        assert_eq!(core.zoom().await, Err(DomainError::NotRunning));
        assert_eq!(core.capabilities().await, Err(DomainError::NotRunning));
    }

    #[tokio::test]
    async fn routes_supported_operations_and_keeps_lifecycle_idempotent() {
        let (core, device) = service_with_supported_ports(false);

        core.start().await.expect("first start must succeed");
        core.start().await.expect("second start must be idempotent");
        assert_eq!(device.open_count.load(Ordering::SeqCst), 1);

        core.set_zoom(Zoom::new(42).expect("valid zoom"))
            .await
            .expect("zoom write must succeed");
        assert_eq!(
            core.zoom().await.expect("zoom read must succeed").value(),
            42
        );

        core.go_to_max_zoom().await.expect("max zoom must succeed");
        assert_eq!(
            core.zoom().await.expect("zoom read must succeed"),
            Zoom::MAX
        );
        core.go_to_min_zoom().await.expect("min zoom must succeed");
        assert_eq!(
            core.zoom().await.expect("zoom read must succeed"),
            Zoom::MIN
        );

        core.set_focus(Focus::new(37).expect("valid focus"))
            .await
            .expect("focus write must succeed");
        assert_eq!(
            core.focus().await.expect("focus read must succeed").value(),
            37
        );
        assert_eq!(
            core.info().await.expect("info must succeed").as_str(),
            "test camera"
        );

        core.stop().await.expect("first stop must succeed");
        core.stop().await.expect("second stop must be idempotent");
        assert_eq!(device.close_count.load(Ordering::SeqCst), 1);
        assert_eq!(core.start().await, Err(DomainError::NotRunning));
    }

    #[tokio::test]
    async fn reports_missing_optional_capabilities_as_unsupported() {
        let (core, _) = service_with_supported_ports(false);
        core.start().await.expect("start must succeed");

        assert_eq!(
            core.set_auto_focus(true).await,
            Err(DomainError::UnsupportedCapability("autofocus"))
        );
        assert_eq!(
            core.auto_focus_enabled().await,
            Err(DomainError::UnsupportedCapability("autofocus"))
        );
        assert_eq!(
            core.set_stabilization(true).await,
            Err(DomainError::UnsupportedCapability("stabilization"))
        );
        assert_eq!(
            core.stabilization_enabled().await,
            Err(DomainError::UnsupportedCapability("stabilization"))
        );
    }

    #[tokio::test]
    async fn shutdown_waits_for_an_active_capability_operation() {
        let (core, device) = service_with_supported_ports(true);
        core.start().await.expect("start must succeed");

        let operation_core = Arc::clone(&core);
        let operation = tokio::spawn(async move {
            operation_core
                .set_zoom(Zoom::new(55).expect("valid zoom"))
                .await
        });
        device.zoom_started.notified().await;

        let shutdown_core = Arc::clone(&core);
        let shutdown = tokio::spawn(async move { shutdown_core.stop().await });
        tokio::task::yield_now().await;
        assert_eq!(device.close_count.load(Ordering::SeqCst), 0);

        device.zoom_release.notify_one();
        operation
            .await
            .expect("operation task must join")
            .expect("operation must succeed");
        shutdown
            .await
            .expect("shutdown task must join")
            .expect("shutdown must succeed");
        assert_eq!(device.close_count.load(Ordering::SeqCst), 1);
    }
}
