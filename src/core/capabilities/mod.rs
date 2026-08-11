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

/// A device feature that is available through a registered core port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    /// Normalized optical or digital zoom control.
    Zoom,
    /// Normalized focus control.
    Focus,
    /// Automatic focus control.
    AutoFocus,
    /// Human-readable device information.
    Info,
    /// Image-stabilization control.
    Stabilization,
}
