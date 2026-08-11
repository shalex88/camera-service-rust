#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use camera_controller_rust::app::Application;
use camera_controller_rust::config::Config;
use clap::Parser;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Parser)]
#[command(version, about = "An extensible camera control service")]
struct Cli {
    #[arg(short, long, value_name = "FILE", default_value = "config/config.yaml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load(&cli.config)
        .with_context(|| format!("failed to load configuration '{}'", cli.config.display()))?;
    let application = Application::from_config(config);
    application.initialize_tracing()?;
    tracing::info!(config = %cli.config.display(), "configuration loaded");

    let cancellation = CancellationToken::new();
    let signal_cancellation = cancellation.clone();
    let signal_task = tokio::spawn(async move {
        match wait_for_shutdown().await {
            Ok(()) => tracing::info!("shutdown signal received"),
            Err(error) => tracing::error!(%error, "failed to listen for shutdown signal"),
        }
        signal_cancellation.cancel();
    });

    let result = application.run(cancellation).await;
    signal_task.abort();
    result
}

#[cfg(unix)]
async fn wait_for_shutdown() -> std::io::Result<()> {
    use tokio::signal::unix::{SignalKind, signal};

    let mut terminate = signal(SignalKind::terminate())?;
    tokio::select! {
        result = tokio::signal::ctrl_c() => result,
        _ = terminate.recv() => Ok(()),
    }
}

#[cfg(not(unix))]
async fn wait_for_shutdown() -> std::io::Result<()> {
    tokio::signal::ctrl_c().await
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::Parser;

    use super::Cli;

    #[test]
    fn cli_uses_the_development_config_by_default() {
        let cli =
            Cli::try_parse_from(["camera-controller-rust"]).expect("default arguments must parse");

        assert_eq!(cli.config, PathBuf::from("config/config.yaml"));
    }

    #[test]
    fn cli_accepts_a_custom_configuration_path() {
        let cli = Cli::try_parse_from(["camera-controller-rust", "--config", "/tmp/camera.yaml"])
            .expect("custom configuration path must parse");

        assert_eq!(cli.config, PathBuf::from("/tmp/camera.yaml"));
    }
}
