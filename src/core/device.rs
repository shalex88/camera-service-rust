use std::sync::Arc;

use async_trait::async_trait;

use crate::core::{
    AutoFocusCapability, Capability, DomainError, FocusCapability, InfoCapability,
    StabilizationCapability, ZoomCapability,
};

#[async_trait]
pub trait DeviceLifecycle: Send + Sync {
    async fn open(&self) -> Result<(), DomainError>;
    async fn close(&self) -> Result<(), DomainError>;
}

#[derive(Clone)]
pub struct DevicePorts {
    lifecycle: Arc<dyn DeviceLifecycle>,
    zoom: Option<Arc<dyn ZoomCapability>>,
    focus: Option<Arc<dyn FocusCapability>>,
    info: Option<Arc<dyn InfoCapability>>,
    autofocus: Option<Arc<dyn AutoFocusCapability>>,
    stabilization: Option<Arc<dyn StabilizationCapability>>,
}

impl DevicePorts {
    pub fn builder(lifecycle: Arc<dyn DeviceLifecycle>) -> DevicePortsBuilder {
        DevicePortsBuilder {
            lifecycle,
            zoom: None,
            focus: None,
            info: None,
            autofocus: None,
            stabilization: None,
        }
    }

    pub fn lifecycle(&self) -> &Arc<dyn DeviceLifecycle> {
        &self.lifecycle
    }

    pub fn zoom(&self) -> Option<&Arc<dyn ZoomCapability>> {
        self.zoom.as_ref()
    }

    pub fn focus(&self) -> Option<&Arc<dyn FocusCapability>> {
        self.focus.as_ref()
    }

    pub fn info(&self) -> Option<&Arc<dyn InfoCapability>> {
        self.info.as_ref()
    }

    pub fn autofocus(&self) -> Option<&Arc<dyn AutoFocusCapability>> {
        self.autofocus.as_ref()
    }

    pub fn stabilization(&self) -> Option<&Arc<dyn StabilizationCapability>> {
        self.stabilization.as_ref()
    }

    pub fn capabilities(&self) -> Vec<Capability> {
        let mut capabilities = Vec::with_capacity(5);
        if self.zoom.is_some() {
            capabilities.push(Capability::Zoom);
        }
        if self.focus.is_some() {
            capabilities.push(Capability::Focus);
        }
        if self.autofocus.is_some() {
            capabilities.push(Capability::AutoFocus);
        }
        if self.info.is_some() {
            capabilities.push(Capability::Info);
        }
        if self.stabilization.is_some() {
            capabilities.push(Capability::Stabilization);
        }
        capabilities
    }
}

pub struct DevicePortsBuilder {
    lifecycle: Arc<dyn DeviceLifecycle>,
    zoom: Option<Arc<dyn ZoomCapability>>,
    focus: Option<Arc<dyn FocusCapability>>,
    info: Option<Arc<dyn InfoCapability>>,
    autofocus: Option<Arc<dyn AutoFocusCapability>>,
    stabilization: Option<Arc<dyn StabilizationCapability>>,
}

impl DevicePortsBuilder {
    pub fn with_zoom(mut self, zoom: Arc<dyn ZoomCapability>) -> Self {
        self.zoom = Some(zoom);
        self
    }

    pub fn with_focus(mut self, focus: Arc<dyn FocusCapability>) -> Self {
        self.focus = Some(focus);
        self
    }

    pub fn with_info(mut self, info: Arc<dyn InfoCapability>) -> Self {
        self.info = Some(info);
        self
    }

    pub fn with_autofocus(mut self, autofocus: Arc<dyn AutoFocusCapability>) -> Self {
        self.autofocus = Some(autofocus);
        self
    }

    pub fn with_stabilization(mut self, stabilization: Arc<dyn StabilizationCapability>) -> Self {
        self.stabilization = Some(stabilization);
        self
    }

    pub fn build(self) -> DevicePorts {
        DevicePorts {
            lifecycle: self.lifecycle,
            zoom: self.zoom,
            focus: self.focus,
            info: self.info,
            autofocus: self.autofocus,
            stabilization: self.stabilization,
        }
    }
}
