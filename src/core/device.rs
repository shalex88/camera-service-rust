use std::sync::Arc;

use async_trait::async_trait;

use crate::core::{
    AutoFocusCapability, Capability, DomainError, FocusCapability, InfoCapability,
    StabilizationCapability, ZoomCapability,
};

/// The required lifecycle port implemented by every device adapter.
#[async_trait]
pub trait DeviceLifecycle: Send + Sync {
    /// Opens the device and prepares its capability ports.
    async fn open(&self) -> Result<(), DomainError>;
    /// Closes the device after active operations have drained.
    async fn close(&self) -> Result<(), DomainError>;
}

/// The lifecycle port and optional capability ports for one device instance.
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
    /// Starts a typed port builder with the required lifecycle port.
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

    /// Returns the required lifecycle port.
    pub fn lifecycle(&self) -> &Arc<dyn DeviceLifecycle> {
        &self.lifecycle
    }

    /// Returns the zoom port when supported.
    pub fn zoom(&self) -> Option<&Arc<dyn ZoomCapability>> {
        self.zoom.as_ref()
    }

    /// Returns the focus port when supported.
    pub fn focus(&self) -> Option<&Arc<dyn FocusCapability>> {
        self.focus.as_ref()
    }

    /// Returns the information port when supported.
    pub fn info(&self) -> Option<&Arc<dyn InfoCapability>> {
        self.info.as_ref()
    }

    /// Returns the autofocus port when supported.
    pub fn autofocus(&self) -> Option<&Arc<dyn AutoFocusCapability>> {
        self.autofocus.as_ref()
    }

    /// Returns the stabilization port when supported.
    pub fn stabilization(&self) -> Option<&Arc<dyn StabilizationCapability>> {
        self.stabilization.as_ref()
    }

    /// Derives the supported capability set from registered ports.
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

/// A builder that registers only the capabilities implemented by a device.
pub struct DevicePortsBuilder {
    lifecycle: Arc<dyn DeviceLifecycle>,
    zoom: Option<Arc<dyn ZoomCapability>>,
    focus: Option<Arc<dyn FocusCapability>>,
    info: Option<Arc<dyn InfoCapability>>,
    autofocus: Option<Arc<dyn AutoFocusCapability>>,
    stabilization: Option<Arc<dyn StabilizationCapability>>,
}

impl DevicePortsBuilder {
    /// Registers a zoom port.
    pub fn with_zoom(mut self, zoom: Arc<dyn ZoomCapability>) -> Self {
        self.zoom = Some(zoom);
        self
    }

    /// Registers a focus port.
    pub fn with_focus(mut self, focus: Arc<dyn FocusCapability>) -> Self {
        self.focus = Some(focus);
        self
    }

    /// Registers an information port.
    pub fn with_info(mut self, info: Arc<dyn InfoCapability>) -> Self {
        self.info = Some(info);
        self
    }

    /// Registers an autofocus port.
    pub fn with_autofocus(mut self, autofocus: Arc<dyn AutoFocusCapability>) -> Self {
        self.autofocus = Some(autofocus);
        self
    }

    /// Registers a stabilization port.
    pub fn with_stabilization(mut self, stabilization: Arc<dyn StabilizationCapability>) -> Self {
        self.stabilization = Some(stabilization);
        self
    }

    /// Finishes the immutable device-port aggregate.
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
