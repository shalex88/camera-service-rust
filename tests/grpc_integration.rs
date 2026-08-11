use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use camera_controller_rust::api::grpc::proto::camera_service_client::CameraServiceClient;
use camera_controller_rust::api::grpc::proto::{
    SetAutoFocusRequest, SetFocusRequest, SetStabilizationRequest, SetZoomRequest,
};
use camera_controller_rust::app::Application;
use camera_controller_rust::config::Config;
use tempfile::NamedTempFile;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tonic::Code;
use tonic::transport::{Channel, Endpoint};

const CONFIG: &str = r#"app:
  name: camera-controller-rust
  log_level: info
  api:
    api_type: grpc
    port: 50051
  core:
    device_type: camera
  infrastructure:
    device_name: fake_simple
"#;

struct RunningService {
    client: CameraServiceClient<Channel>,
    cancellation: CancellationToken,
    task: JoinHandle<anyhow::Result<()>>,
}

impl RunningService {
    async fn start() -> Self {
        let file = NamedTempFile::new().expect("test must create configuration file");
        fs::write(file.path(), CONFIG).expect("test must write configuration");
        let config = Config::load(file.path()).expect("test configuration must load");
        let application = Application::from_config(config);
        let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
            .await
            .expect("test listener must bind");
        let address = listener
            .local_addr()
            .expect("listener must have an address");
        let cancellation = CancellationToken::new();
        let run_cancellation = cancellation.clone();
        let task = tokio::spawn(async move {
            application
                .run_with_listener(listener, run_cancellation)
                .await
        });
        let client = connect_with_retry(address).await;
        Self {
            client,
            cancellation,
            task,
        }
    }

    async fn shutdown(self) {
        self.cancellation.cancel();
        self.task
            .await
            .expect("application task must join")
            .expect("application must shut down cleanly");
    }
}

async fn connect_with_retry(address: SocketAddr) -> CameraServiceClient<Channel> {
    let endpoint = Endpoint::from_shared(format!("http://{address}"))
        .expect("loopback endpoint must be valid");
    for _ in 0..50 {
        if let Ok(channel) = endpoint.connect().await {
            return CameraServiceClient::new(channel);
        }
        sleep(Duration::from_millis(10)).await;
    }
    panic!("server did not become ready at {address}");
}

#[tokio::test]
async fn supported_rpc_operations_round_trip_over_grpc() {
    let mut service = RunningService::start().await;

    service
        .client
        .set_zoom(SetZoomRequest { zoom: 73 })
        .await
        .expect("set zoom must succeed");
    assert_eq!(
        service
            .client
            .get_zoom(())
            .await
            .expect("get zoom must succeed")
            .into_inner()
            .zoom,
        73
    );
    service
        .client
        .go_to_max_zoom(())
        .await
        .expect("max zoom must succeed");
    assert_eq!(
        service
            .client
            .get_zoom(())
            .await
            .expect("get max zoom must succeed")
            .into_inner()
            .zoom,
        100
    );
    service
        .client
        .go_to_min_zoom(())
        .await
        .expect("min zoom must succeed");
    assert_eq!(
        service
            .client
            .get_zoom(())
            .await
            .expect("get min zoom must succeed")
            .into_inner()
            .zoom,
        0
    );

    service
        .client
        .set_focus(SetFocusRequest { focus: 29 })
        .await
        .expect("set focus must succeed");
    assert_eq!(
        service
            .client
            .get_focus(())
            .await
            .expect("get focus must succeed")
            .into_inner()
            .focus,
        29
    );
    assert_eq!(
        service
            .client
            .get_info(())
            .await
            .expect("get info must succeed")
            .into_inner()
            .info,
        "Fake Simple Camera"
    );
    assert_eq!(
        service
            .client
            .get_capabilities(())
            .await
            .expect("capabilities must succeed")
            .into_inner()
            .capabilities,
        [1, 2, 4]
    );

    service.shutdown().await;
}

#[tokio::test]
async fn invalid_normalized_values_are_invalid_arguments_over_grpc() {
    let mut service = RunningService::start().await;

    let zoom = service
        .client
        .set_zoom(SetZoomRequest { zoom: 101 })
        .await
        .expect_err("invalid zoom must fail");
    let focus = service
        .client
        .set_focus(SetFocusRequest { focus: 101 })
        .await
        .expect_err("invalid focus must fail");

    assert_eq!(zoom.code(), Code::InvalidArgument);
    assert_eq!(focus.code(), Code::InvalidArgument);
    service.shutdown().await;
}

#[tokio::test]
async fn unsupported_capabilities_are_unimplemented_over_grpc() {
    let mut service = RunningService::start().await;

    let errors = [
        service
            .client
            .set_auto_focus(SetAutoFocusRequest { enable: true })
            .await
            .expect_err("set autofocus must be unsupported"),
        service
            .client
            .get_auto_focus(())
            .await
            .expect_err("get autofocus must be unsupported"),
        service
            .client
            .set_stabilization(SetStabilizationRequest { enable: true })
            .await
            .expect_err("set stabilization must be unsupported"),
        service
            .client
            .get_stabilization(())
            .await
            .expect_err("get stabilization must be unsupported"),
    ];

    assert!(
        errors
            .iter()
            .all(|error| error.code() == Code::Unimplemented)
    );
    service.shutdown().await;
}

#[tokio::test]
async fn concurrent_grpc_writes_leave_normalized_state() {
    let service = RunningService::start().await;
    let mut tasks = Vec::new();

    for value in 0..=100 {
        let mut client = service.client.clone();
        tasks.push(tokio::spawn(async move {
            client.set_zoom(SetZoomRequest { zoom: value }).await?;
            client
                .set_focus(SetFocusRequest { focus: 100 - value })
                .await?;
            Ok::<(), tonic::Status>(())
        }));
    }
    for task in tasks {
        task.await
            .expect("client task must join")
            .expect("normalized write must succeed");
    }

    let mut client = service.client.clone();
    assert!(
        client
            .get_zoom(())
            .await
            .expect("zoom must read")
            .into_inner()
            .zoom
            <= 100
    );
    assert!(
        client
            .get_focus(())
            .await
            .expect("focus must read")
            .into_inner()
            .focus
            <= 100
    );
    service.shutdown().await;
}
