use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    app: AppConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppConfig {
    name: String,
    log_level: LogLevel,
    api: ApiConfig,
    core: CoreConfig,
    infrastructure: InfrastructureConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApiConfig {
    api_type: ApiType,
    port: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoreConfig {
    device_type: DeviceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct InfrastructureConfig {
    device_name: DeviceName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiType {
    Grpc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Camera,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceName {
    FakeSimple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read configuration '{path}': {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid configuration at {path}: {message}")]
    Deserialize { path: String, message: String },
    #[error("invalid configuration at {path}: {message}")]
    Validation {
        path: &'static str,
        message: &'static str,
    },
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let deserializer = serde_yaml_ng::Deserializer::from_str(&contents);
        let config: Self = serde_path_to_error::deserialize(deserializer).map_err(|error| {
            ConfigError::Deserialize {
                path: error.path().to_string(),
                message: error.inner().to_string(),
            }
        })?;

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.app.name.trim().is_empty() {
            return Err(ConfigError::Validation {
                path: "app.name",
                message: "must be nonempty after trimming",
            });
        }
        if self.app.api.port == 0 {
            return Err(ConfigError::Validation {
                path: "app.api.port",
                message: "must be in 1..=65535",
            });
        }
        Ok(())
    }

    pub fn app_name(&self) -> &str {
        &self.app.name
    }

    pub const fn log_level(&self) -> &'static str {
        self.app.log_level.as_str()
    }

    pub const fn api_type(&self) -> ApiType {
        self.app.api.api_type
    }

    pub const fn api_port(&self) -> u16 {
        self.app.api.port
    }

    pub const fn device_type(&self) -> DeviceType {
        self.app.core.device_type
    }

    pub const fn device_name(&self) -> DeviceName {
        self.app.infrastructure.device_name
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use super::{ApiType, Config, DeviceName, DeviceType};

    const VALID_YAML: &str = r#"app:
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

    fn load_yaml(yaml: &str) -> Result<Config, super::ConfigError> {
        let file = NamedTempFile::new().expect("test must create a temporary file");
        fs::write(file.path(), yaml).expect("test must write YAML fixture");
        Config::load(file.path())
    }

    #[test]
    fn loads_the_approved_yaml_as_typed_configuration() {
        let config = load_yaml(VALID_YAML).expect("approved YAML must load");

        assert_eq!(config.app_name(), "camera-controller-rust");
        assert_eq!(config.log_level(), "info");
        assert_eq!(config.api_type(), ApiType::Grpc);
        assert_eq!(config.api_port(), 50051);
        assert_eq!(config.device_type(), DeviceType::Camera);
        assert_eq!(config.device_name(), DeviceName::FakeSimple);
    }

    #[test]
    fn rejects_an_unknown_nested_field_with_its_path() {
        let yaml = VALID_YAML.replace("    port: 50051", "    port: 50051\n    host: localhost");

        let error = load_yaml(&yaml).expect_err("unknown API field must fail");

        assert!(error.to_string().contains("app.api"));
        assert!(error.to_string().contains("host"));
    }

    #[test]
    fn rejects_a_missing_nested_field_with_its_path() {
        let yaml = VALID_YAML.replace("    port: 50051\n", "");

        let error = load_yaml(&yaml).expect_err("missing port must fail");

        assert!(error.to_string().contains("app.api"));
        assert!(error.to_string().contains("port"));
    }

    #[test]
    fn rejects_an_empty_application_name() {
        let yaml = VALID_YAML.replace("camera-controller-rust", "   ");

        let error = load_yaml(&yaml).expect_err("blank application name must fail");

        assert!(error.to_string().contains("app.name"));
        assert!(error.to_string().contains("nonempty"));
    }

    #[test]
    fn rejects_port_zero() {
        let yaml = VALID_YAML.replace("50051", "0");

        let error = load_yaml(&yaml).expect_err("port zero must fail");

        assert!(error.to_string().contains("app.api.port"));
        assert!(error.to_string().contains("1..=65535"));
    }

    #[test]
    fn reports_the_path_for_an_unreadable_configuration_file() {
        let error = Config::load("/path/that/does/not/exist/config.yaml")
            .expect_err("missing file must fail");

        assert!(
            error
                .to_string()
                .contains("/path/that/does/not/exist/config.yaml")
        );
    }
}
