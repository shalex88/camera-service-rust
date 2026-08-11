mod builder;
mod devices;

pub use builder::build_device;
pub use devices::SimpleFakeCamera;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::config::DeviceName;
    use crate::core::{
        Capability, DeviceLifecycle, DomainError, Focus, FocusCapability, InfoCapability, Zoom,
        ZoomCapability,
    };

    use super::{SimpleFakeCamera, build_device};

    #[tokio::test]
    async fn fake_rejects_capability_calls_while_closed() {
        let camera = SimpleFakeCamera::new();

        assert!(matches!(
            camera.zoom().await,
            Err(DomainError::DeviceUnavailable(_))
        ));
        assert!(matches!(
            camera.info().await,
            Err(DomainError::DeviceUnavailable(_))
        ));
    }

    #[tokio::test]
    async fn fake_has_independent_normalized_state_and_idempotent_lifecycle() {
        let first = SimpleFakeCamera::new();
        let second = SimpleFakeCamera::new();

        first.open().await.expect("first open must succeed");
        first.open().await.expect("second open must be idempotent");
        second.open().await.expect("second instance must open");

        assert_eq!(first.zoom().await.expect("zoom must read"), Zoom::MIN);
        assert_eq!(first.focus().await.expect("focus must read"), Focus::MIN);
        assert_eq!(
            first.info().await.expect("info must read").as_str(),
            "Fake Simple Camera"
        );

        first
            .set_zoom(Zoom::new(27).expect("valid zoom"))
            .await
            .expect("zoom must write");
        first
            .set_focus(Focus::new(63).expect("valid focus"))
            .await
            .expect("focus must write");

        assert_eq!(first.zoom().await.expect("zoom must read").value(), 27);
        assert_eq!(first.focus().await.expect("focus must read").value(), 63);
        assert_eq!(second.zoom().await.expect("zoom must read"), Zoom::MIN);
        assert_eq!(second.focus().await.expect("focus must read"), Focus::MIN);

        first.close().await.expect("first close must succeed");
        first
            .close()
            .await
            .expect("second close must be idempotent");
        assert!(matches!(
            first.focus().await,
            Err(DomainError::DeviceUnavailable(_))
        ));
    }

    #[tokio::test]
    async fn fake_state_remains_valid_under_concurrent_writes() {
        let camera = Arc::new(SimpleFakeCamera::new());
        camera.open().await.expect("open must succeed");

        let mut tasks = Vec::new();
        for value in 0..=100 {
            let camera = Arc::clone(&camera);
            tasks.push(tokio::spawn(async move {
                camera
                    .set_zoom(Zoom::new(value).expect("loop value is normalized"))
                    .await
            }));
        }
        for task in tasks {
            task.await
                .expect("writer task must join")
                .expect("writer must succeed");
        }

        assert!(camera.zoom().await.expect("zoom must read").value() <= 100);
    }

    #[test]
    fn builder_registers_only_simple_fake_capabilities() {
        let ports = build_device(DeviceName::FakeSimple);

        assert_eq!(
            ports.capabilities(),
            vec![Capability::Zoom, Capability::Focus, Capability::Info]
        );
        assert!(ports.autofocus().is_none());
        assert!(ports.stabilization().is_none());
    }
}
