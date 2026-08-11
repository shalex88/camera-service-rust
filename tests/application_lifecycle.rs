use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use camera_controller_rust::app::Application;
use camera_controller_rust::config::Config;
use tempfile::NamedTempFile;
use tokio::net::TcpListener;
use tokio::time::{sleep, timeout};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tonic::Request;
use tonic::transport::{Channel, Endpoint};
use tonic_health::ServingStatus;
use tonic_health::pb::HealthCheckRequest;
use tonic_health::pb::health_client::HealthClient;
use tonic_reflection::pb::v1::ServerReflectionRequest;
use tonic_reflection::pb::v1::server_reflection_client::ServerReflectionClient;
use tonic_reflection::pb::v1::server_reflection_request::MessageRequest;
use tonic_reflection::pb::v1::server_reflection_response::MessageResponse;

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

async fn connect_with_retry(address: SocketAddr) -> Channel {
    let endpoint = Endpoint::from_shared(format!("http://{address}"))
        .expect("loopback endpoint must be valid");
    for _ in 0..50 {
        if let Ok(channel) = endpoint.connect().await {
            return channel;
        }
        sleep(Duration::from_millis(10)).await;
    }
    panic!("server did not become ready at {address}");
}

#[tokio::test]
async fn application_closes_health_watches_before_releasing_its_listener() {
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
    let application_task = tokio::spawn(async move {
        application
            .run_with_listener(listener, run_cancellation)
            .await
    });

    let channel = connect_with_retry(address).await;
    let mut health = HealthClient::new(channel.clone());
    let health_response = health
        .check(Request::new(HealthCheckRequest {
            service: "camera.v1.CameraService".to_owned(),
        }))
        .await
        .expect("health request must succeed")
        .into_inner();
    assert_eq!(health_response.status, ServingStatus::Serving as i32);

    let mut health_updates = health
        .watch(Request::new(HealthCheckRequest {
            service: "camera.v1.CameraService".to_owned(),
        }))
        .await
        .expect("health watch must start")
        .into_inner();
    let serving_update = health_updates
        .message()
        .await
        .expect("health watch must remain valid")
        .expect("health watch must report its initial state");
    assert_eq!(serving_update.status, ServingStatus::Serving as i32);

    let mut overall_health_updates = health
        .watch(Request::new(HealthCheckRequest {
            service: String::new(),
        }))
        .await
        .expect("overall health watch must start")
        .into_inner();
    let overall_serving_update = overall_health_updates
        .message()
        .await
        .expect("overall health watch must remain valid")
        .expect("overall health watch must report its initial state");
    assert_eq!(overall_serving_update.status, ServingStatus::Serving as i32);

    let mut reflection = ServerReflectionClient::new(channel);
    let reflection_request = ServerReflectionRequest {
        host: String::new(),
        message_request: Some(MessageRequest::ListServices(String::new())),
    };
    let mut responses = reflection
        .server_reflection_info(Request::new(tokio_stream::iter([reflection_request])))
        .await
        .expect("reflection request must succeed")
        .into_inner();
    let response = responses
        .next()
        .await
        .expect("reflection must return one response")
        .expect("reflection response must be successful");
    let Some(MessageResponse::ListServicesResponse(services)) = response.message_response else {
        panic!("reflection must return a service list");
    };
    assert!(
        services
            .service
            .iter()
            .any(|service| service.name == "camera.v1.CameraService")
    );

    cancellation.cancel();
    let shutdown_update = timeout(Duration::from_secs(1), health_updates.message())
        .await
        .expect("health watch must report shutdown promptly")
        .expect("health watch must remain valid during shutdown")
        .expect("health watch must report a shutdown state");
    assert_eq!(shutdown_update.status, ServingStatus::NotServing as i32);
    let overall_shutdown_update = timeout(Duration::from_secs(1), overall_health_updates.message())
        .await
        .expect("overall health watch must report shutdown promptly")
        .expect("overall health watch must remain valid during shutdown")
        .expect("overall health watch must report a shutdown state");
    assert_eq!(
        overall_shutdown_update.status,
        ServingStatus::NotServing as i32
    );
    timeout(Duration::from_secs(1), application_task)
        .await
        .expect("application shutdown must not wait for the health client")
        .expect("application task must join")
        .expect("application must shut down cleanly");
    let stream_end = timeout(Duration::from_secs(1), health_updates.message())
        .await
        .expect("health watch must close promptly")
        .expect("health watch must remain valid while closing");
    assert!(stream_end.is_none());
    let overall_stream_end = timeout(Duration::from_secs(1), overall_health_updates.message())
        .await
        .expect("overall health watch must close promptly")
        .expect("overall health watch must remain valid while closing");
    assert!(overall_stream_end.is_none());

    TcpListener::bind(address)
        .await
        .expect("application must release its listening address");
}
