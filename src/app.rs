use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::EnvFilter;

use crate::api::grpc::GrpcServer;
use crate::config::{ApiType, Config, DeviceType};
use crate::core::CameraCore;
use crate::infrastructure::build_device;

/// The composition root for configuration, device, core, and API lifecycle.
pub struct Application {
    config: Config,
    core: Arc<CameraCore>,
}

impl Application {
    /// Builds the selected device ports and core from validated configuration.
    pub fn from_config(config: Config) -> Self {
        match config.device_type() {
            DeviceType::Camera => {}
        }
        let ports = build_device(config.device_name());
        Self {
            config,
            core: Arc::new(CameraCore::new(ports)),
        }
    }

    /// Installs structured logging with the configured level.
    pub fn initialize_tracing(&self) -> Result<()> {
        let filter = EnvFilter::try_new(self.config.log_level())
            .context("failed to create tracing filter")?;
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .try_init()
            .map_err(|error| anyhow::anyhow!("failed to initialize tracing subscriber: {error}"))
    }

    /// Binds the configured loopback address and runs until cancellation.
    pub async fn run(self, cancellation: CancellationToken) -> Result<()> {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), self.config.api_port());
        let listener = TcpListener::bind(address)
            .await
            .with_context(|| format!("failed to bind gRPC listener at {address}"))?;
        self.run_with_listener(listener, cancellation).await
    }

    /// Runs with a caller-provided listener, primarily for embedding and tests.
    pub async fn run_with_listener(
        self,
        listener: TcpListener,
        cancellation: CancellationToken,
    ) -> Result<()> {
        let local_address = listener
            .local_addr()
            .context("failed to read gRPC listener address")?;
        self.core
            .start()
            .await
            .context("failed to start camera core")?;

        tracing::info!(
            application = self.config.app_name(),
            version = env!("CARGO_PKG_VERSION"),
            address = %local_address,
            "camera service started"
        );

        let server_result = match self.config.api_type() {
            ApiType::Grpc => GrpcServer::new(Arc::clone(&self.core))
                .serve(listener, cancellation)
                .await
                .context("gRPC runtime stopped with an error"),
        };
        let stop_result = self.core.stop().await.context("failed to stop camera core");

        server_result?;
        stop_result?;
        tracing::info!("camera service stopped");
        Ok(())
    }
}
