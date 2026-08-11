mod autofocus;
mod focus;
mod info;
mod stabilization;
mod zoom;

pub use autofocus::AutoFocusCapability;
pub use focus::{Focus, FocusCapability};
pub use info::{DeviceInfo, InfoCapability};
pub use stabilization::StabilizationCapability;
pub use zoom::{Zoom, ZoomCapability};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Zoom,
    Focus,
    AutoFocus,
    Info,
    Stabilization,
}
