use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::core::{
    DeviceInfo, DeviceLifecycle, DomainError, Focus, FocusCapability, InfoCapability, Zoom,
    ZoomCapability,
};

#[derive(Debug)]
struct FakeState {
    open: bool,
    zoom: Zoom,
    focus: Focus,
}

impl Default for FakeState {
    fn default() -> Self {
        Self {
            open: false,
            zoom: Zoom::MIN,
            focus: Focus::MIN,
        }
    }
}

/// A concurrency-safe, per-instance camera adapter backed only by memory.
#[derive(Debug, Default)]
pub struct SimpleFakeCamera {
    state: RwLock<FakeState>,
}

impl SimpleFakeCamera {
    /// Creates a closed fake camera with minimum zoom and focus.
    pub fn new() -> Self {
        Self::default()
    }

    fn ensure_open(open: bool) -> Result<(), DomainError> {
        if open {
            Ok(())
        } else {
            Err(DomainError::DeviceUnavailable(
                "simple fake camera is closed".to_owned(),
            ))
        }
    }
}

#[async_trait]
impl DeviceLifecycle for SimpleFakeCamera {
    async fn open(&self) -> Result<(), DomainError> {
        self.state.write().await.open = true;
        Ok(())
    }

    async fn close(&self) -> Result<(), DomainError> {
        self.state.write().await.open = false;
        Ok(())
    }
}

#[async_trait]
impl ZoomCapability for SimpleFakeCamera {
    async fn set_zoom(&self, zoom: Zoom) -> Result<(), DomainError> {
        let mut state = self.state.write().await;
        Self::ensure_open(state.open)?;
        state.zoom = zoom;
        Ok(())
    }

    async fn zoom(&self) -> Result<Zoom, DomainError> {
        let state = self.state.read().await;
        Self::ensure_open(state.open)?;
        Ok(state.zoom)
    }
}

#[async_trait]
impl FocusCapability for SimpleFakeCamera {
    async fn set_focus(&self, focus: Focus) -> Result<(), DomainError> {
        let mut state = self.state.write().await;
        Self::ensure_open(state.open)?;
        state.focus = focus;
        Ok(())
    }

    async fn focus(&self) -> Result<Focus, DomainError> {
        let state = self.state.read().await;
        Self::ensure_open(state.open)?;
        Ok(state.focus)
    }
}

#[async_trait]
impl InfoCapability for SimpleFakeCamera {
    async fn info(&self) -> Result<DeviceInfo, DomainError> {
        let state = self.state.read().await;
        Self::ensure_open(state.open)?;
        Ok(DeviceInfo::new("Fake Simple Camera"))
    }
}
