use std::sync::Arc;

use crate::config::DeviceName;
use crate::core::{DeviceLifecycle, DevicePorts, FocusCapability, InfoCapability, ZoomCapability};
use crate::infrastructure::devices::SimpleFakeCamera;

pub fn build_device(device_name: DeviceName) -> DevicePorts {
    match device_name {
        DeviceName::FakeSimple => {
            let camera = Arc::new(SimpleFakeCamera::new());
            let lifecycle: Arc<dyn DeviceLifecycle> = camera.clone();
            let zoom: Arc<dyn ZoomCapability> = camera.clone();
            let focus: Arc<dyn FocusCapability> = camera.clone();
            let info: Arc<dyn InfoCapability> = camera;

            DevicePorts::builder(lifecycle)
                .with_zoom(zoom)
                .with_focus(focus)
                .with_info(info)
                .build()
        }
    }
}
