use std::sync::Arc;

use thiserror::Error;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;

use crate::api::grpc::proto::camera_service_server::CameraServiceServer;
use crate::api::grpc::{FILE_DESCRIPTOR_SET, GrpcCameraService};
use crate::core::CameraCore;

#[derive(Debug, Error)]
pub enum GrpcServerError {
    #[error("failed to build gRPC reflection service: {0}")]
    Reflection(String),
    #[error("gRPC server failed: {0}")]
    Transport(#[from] tonic::transport::Error),
}

pub struct GrpcServer {
    core: Arc<CameraCore>,
}

impl GrpcServer {
    pub fn new(core: Arc<CameraCore>) -> Self {
        Self { core }
    }

    pub async fn serve(
        self,
        listener: TcpListener,
        cancellation: CancellationToken,
    ) -> Result<(), GrpcServerError> {
        let camera_service = GrpcCameraService::new(self.core);
        let camera_server = CameraServiceServer::new(camera_service);
        let (health_reporter, health_server) = tonic_health::server::health_reporter();
        health_reporter
            .set_serving::<CameraServiceServer<GrpcCameraService>>()
            .await;
        let reflection_server = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()
            .map_err(|error| GrpcServerError::Reflection(error.to_string()))?;

        let shutdown_reporter = health_reporter.clone();
        let shutdown = async move {
            cancellation.cancelled().await;
            shutdown_reporter
                .set_not_serving::<CameraServiceServer<GrpcCameraService>>()
                .await;
        };

        Server::builder()
            .add_service(health_server)
            .add_service(reflection_server)
            .add_service(camera_server)
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), shutdown)
            .await?;
        Ok(())
    }
}
