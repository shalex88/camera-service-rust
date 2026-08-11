# Rust Camera Service Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an idiomatic three-layer Rust camera service with the complete `camera.v1.CameraService` gRPC contract and an extensible simple fake-camera adapter.

**Architecture:** A single Cargo package exposes a library and thin binary. The API and infrastructure layers both depend inward on capability-specific core ports; `app.rs` is the composition root. Tonic serves the unchanged protobuf contract on loopback, while typed YAML selects the fake device and gRPC adapter.

**Tech Stack:** Rust 1.96.1, edition 2024, Tokio 1.53, Tonic 0.14, Prost 0.14, Serde with `serde_yaml_ng`, Clap 4.6, `tracing`, `thiserror`, and `async-trait`.

## Global Constraints

- Keep exactly three layers: `api`, `core`, and `infrastructure`; composition belongs in `app.rs`.
- Preserve the complete `camera.v1.CameraService` protobuf package, RPCs, fields, and enum numbers from reference commit `b21212d`.
- Support only `device_type: camera` and `device_name: fake_simple` in the first release.
- Bind to `127.0.0.1`; ports must be in `1..=65535` in application configuration.
- The fake device supports Zoom, Focus, and Info only; AutoFocus and Stabilization return `UNIMPLEMENTED`.
- Use capability-specific async ports; do not introduce a god trait, downcasting, `Any`, global mutable state, or dynamic plugins.
- Forbid unsafe code and do not use `unwrap`, `expect`, or panics in production request, configuration, lifecycle, or device paths.
- Commit `Cargo.lock`; do not commit generated protobuf Rust files or build artifacts.
- Every implementation task follows red-green-refactor and ends with an independently testable commit.

---

### Task 1: Package, protobuf contract, and build pipeline

**Files:**
- Create: `.gitignore`
- Create: `Cargo.toml`
- Create: `build.rs`
- Create: `proto/camera_service.proto`
- Create: `src/lib.rs`
- Create: `src/main.rs`
- Create: `src/api/mod.rs`
- Create: `src/api/grpc/mod.rs`
- Create: `src/core/mod.rs`
- Create: `src/infrastructure/mod.rs`
- Create: `tests/api_contract.rs`

**Interfaces:**
- Produces: `api::grpc::proto`, generated from package `camera.v1`.
- Produces: `api::grpc::FILE_DESCRIPTOR_SET: &'static [u8]` for reflection and contract tests.
- Produces: Cargo library crate `camera_controller_rust` and binary `camera-controller-rust`.

- [ ] **Step 1: Create the Cargo package metadata and dependency policy**

Use package metadata:

```toml
[package]
name = "camera-controller-rust"
version = "0.1.0"
edition = "2024"
rust-version = "1.96"
build = "build.rs"

[lib]
name = "camera_controller_rust"
path = "src/lib.rs"

[[bin]]
name = "camera-controller-rust"
path = "src/main.rs"

[dependencies]
anyhow = "1.0.104"
async-trait = "0.1.92"
clap = { version = "4.6.6", features = ["derive"] }
prost = "0.14.4"
serde = { version = "1.0.229", features = ["derive"] }
serde_path_to_error = "0.1.20"
serde_yaml_ng = "0.10.0"
thiserror = "2.0.20"
tokio = { version = "1.53.1", features = ["full"] }
tokio-stream = { version = "0.1.19", features = ["net"] }
tokio-util = { version = "0.7.19", features = ["rt"] }
tonic = { version = "0.14.6", features = ["transport"] }
tonic-health = "0.14.6"
tonic-prost = "0.14.6"
tonic-reflection = "0.14.6"
tracing = "0.1.44"
tracing-subscriber = { version = "0.3.23", features = ["env-filter"] }

[build-dependencies]
prost-build = "0.14.4"
protoc-bin-vendored = "3.2.0"
tonic-prost-build = "0.14.6"

[dev-dependencies]
prost-types = "0.14.4"
tempfile = "3.27.0"

[lints.rust]
unsafe_code = "forbid"
```

Ignore `/target`, editor files, and local logs, while keeping `Cargo.lock` tracked.

- [ ] **Step 2: Preserve the reference protobuf contract**

Copy `proto/camera_service.proto` byte-for-byte from the reference branch. It must define the 12 existing RPCs and capability enum values `0..=5` without renaming fields or messages.

- [ ] **Step 3: Configure reproducible protobuf generation**

Implement `build.rs` without mutating `PROTOC` in the process environment:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    let include = protoc_bin_vendored::include_path()?;
    let mut prost = prost_build::Config::new();
    prost.protoc_executable(protoc);

    tonic_prost_build::configure()
        .file_descriptor_set_path(
            std::path::PathBuf::from(std::env::var("OUT_DIR")?)
                .join("camera_descriptor.bin"),
        )
        .compile_with_config(prost, &["proto/camera_service.proto"], &["proto", include.to_str().ok_or("non-UTF-8 include path")?])?;

    println!("cargo:rerun-if-changed=proto/camera_service.proto");
    Ok(())
}
```

- [ ] **Step 4: Expose generated API types and the descriptor**

In `src/api/grpc/mod.rs`:

```rust
pub mod proto {
    tonic::include_proto!("camera.v1");
}

pub const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("camera_descriptor");
```

Export `api`, `core`, and `infrastructure` from `src/lib.rs`, create empty module declarations for later files, and add a temporary empty `fn main() {}` so every Cargo target exists from the first commit.

- [ ] **Step 5: Write the protobuf descriptor contract test**

Decode `FILE_DESCRIPTOR_SET` with `prost_types::FileDescriptorSet` and assert:

```rust
assert_eq!(file.package.as_deref(), Some("camera.v1"));
assert_eq!(service.name.as_deref(), Some("CameraService"));
assert_eq!(service.method.len(), 12);
assert_eq!(method_names, [
    "SetZoom", "GetZoom", "GoToMinZoom", "GoToMaxZoom",
    "SetFocus", "GetFocus", "SetAutoFocus", "GetAutoFocus",
    "GetInfo", "GetCapabilities", "SetStabilization", "GetStabilization",
]);
```

Also assert request field numbers and all `Capability` enum numbers.

- [ ] **Step 6: Run the contract test and fix only build-pipeline issues**

Run: `cargo test --test api_contract -- --nocapture`  
Expected: PASS with one descriptor-contract test.

- [ ] **Step 7: Run formatting and commit**

Run: `cargo fmt --all --check && git diff --check`  
Commit:

```bash
git add .gitignore Cargo.toml Cargo.lock build.rs proto src tests/api_contract.rs
git commit -m "build: scaffold Rust gRPC contract"
```

### Task 2: Typed configuration and CLI schema

**Files:**
- Create: `config/config.yaml`
- Create: `src/config.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: `Config::load(path: impl AsRef<Path>) -> Result<Config, ConfigError>`.
- Produces: `ApiType::Grpc`, `DeviceType::Camera`, and `DeviceName::FakeSimple`.
- Produces: getters for application name, log level, API port/type, core device type, and infrastructure device name.

- [ ] **Step 1: Write configuration tests first**

Place unit tests in `src/config.rs` covering the exact YAML:

```yaml
app:
  name: camera-controller-rust
  log_level: info
  api:
    api_type: grpc
    port: 50051
  core:
    device_type: camera
  infrastructure:
    device_name: fake_simple
```

Tests must assert successful typed values, unknown nested field rejection, missing field rejection, empty trimmed name rejection, zero port rejection, and path-aware messages such as `app.api.port`.

- [ ] **Step 2: Run configuration tests to verify failure**

Run: `cargo test config::tests -- --nocapture`  
Expected: FAIL because `Config`, enum types, and errors do not exist.

- [ ] **Step 3: Implement typed deserialization and semantic validation**

Use private fields, `#[serde(deny_unknown_fields)]` on every mapping, and snake-case enums:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiType { Grpc }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType { Camera }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceName { FakeSimple }
```

Deserialize with `serde_path_to_error::deserialize(serde_yaml_ng::Deserializer::from_str(&text))`, then call `Config::validate`. Represent I/O, YAML, empty-name, and invalid-port failures as concrete `ConfigError` variants.

- [ ] **Step 4: Run configuration tests**

Run: `cargo test config::tests -- --nocapture`  
Expected: PASS.

- [ ] **Step 5: Add the approved development configuration and commit**

Run: `cargo fmt --all --check && cargo clippy --lib -- -D warnings`  
Commit:

```bash
git add config/config.yaml src/config.rs src/lib.rs
git commit -m "feat: add typed YAML configuration"
```

### Task 3: Core types, errors, and capability ports

**Files:**
- Create: `src/core/error.rs`
- Create: `src/core/capabilities/mod.rs`
- Create: `src/core/capabilities/zoom.rs`
- Create: `src/core/capabilities/focus.rs`
- Create: `src/core/capabilities/info.rs`
- Create: `src/core/capabilities/autofocus.rs`
- Create: `src/core/capabilities/stabilization.rs`
- Create: `src/core/device.rs`
- Modify: `src/core/mod.rs`

**Interfaces:**
- Produces: `Zoom::new(i64)`, `Focus::new(i64)`, `value() -> u8`, and `MIN`/`MAX` constants.
- Produces: `DeviceInfo::new(impl Into<String>)` and `as_str()`.
- Produces: capability traits with the signatures approved in the design.
- Produces: `Capability::{Zoom, Focus, AutoFocus, Info, Stabilization}`.
- Produces: `DevicePorts::builder(lifecycle)` and optional port accessors.

- [ ] **Step 1: Write failing newtype and port-registration tests**

Tests must include:

```rust
assert_eq!(Zoom::new(0)?.value(), 0);
assert_eq!(Zoom::new(100)?.value(), 100);
assert!(matches!(Zoom::new(-1), Err(DomainError::InvalidArgument { .. })));
assert!(matches!(Zoom::new(101), Err(DomainError::InvalidArgument { .. })));
assert_eq!(Focus::new(100)?.value(), 100);
```

Define a test lifecycle adapter, construct `DevicePorts`, and assert that registering zoom/focus/info yields exactly `[Zoom, Focus, Info]`, while missing autofocus and stabilization accessors return `None`.

- [ ] **Step 2: Run core type tests to verify failure**

Run: `cargo test core::capabilities core::device -- --nocapture`  
Expected: FAIL because core types and ports are absent.

- [ ] **Step 3: Implement the typed error model**

In `core/error.rs` define:

```rust
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("invalid {field}: {value}; expected {expected}")]
    InvalidArgument { field: &'static str, value: String, expected: &'static str },
    #[error("capability '{0}' is not supported")]
    UnsupportedCapability(&'static str),
    #[error("camera service is not running")]
    NotRunning,
    #[error("device is unavailable: {0}")]
    DeviceUnavailable(String),
    #[error("device operation failed: {0}")]
    Device(String),
}
```

- [ ] **Step 4: Implement focused async capability traits and domain values**

Keep each trait in its matching module. `Zoom::new(i64)` and `Focus::new(i64)` validate `0..=100`; associated `MIN` and `MAX` constants provide infallible boundary values to core and adapter code. Port traits use `async_trait` and return `Result<_, DomainError>`.

- [ ] **Step 5: Implement `DevicePorts` and its builder**

The builder requires `Arc<dyn DeviceLifecycle>`, defaults every capability to `None`, provides `with_zoom`, `with_focus`, `with_info`, `with_autofocus`, and `with_stabilization`, and derives `capabilities()` in the stable order Zoom, Focus, AutoFocus, Info, Stabilization.

- [ ] **Step 6: Run core type tests and commit**

Run: `cargo test core::capabilities core::device -- --nocapture && cargo clippy --lib -- -D warnings`  
Expected: PASS.  
Commit:

```bash
git add src/core
git commit -m "feat: define capability-specific core ports"
```

### Task 4: Core service and lifecycle coordination

**Files:**
- Create: `src/core/service.rs`
- Modify: `src/core/mod.rs`

**Interfaces:**
- Produces: `CameraCore::new(DevicePorts) -> CameraCore`.
- Produces: async `start`, `stop`, zoom/focus operations, min/max zoom, info, autofocus, stabilization, and `capabilities`.
- Consumes: capability ports and `DomainError` from Task 3.

- [ ] **Step 1: Write failing core-service tests with local test ports**

Tests must prove:

- Calls before `start` return `NotRunning`.
- Repeated `start` and repeated `stop` succeed without duplicate open/close.
- Zoom, focus, and info route to their registered ports.
- Min/max operations pass exact values `0` and `100`.
- Missing autofocus and stabilization ports return `UnsupportedCapability`.
- Shutdown waits for an active operation before invoking close.

Use a `TestDevice` defined inside the test module; core tests must not import infrastructure.

- [ ] **Step 2: Run service tests to verify failure**

Run: `cargo test core::service::tests -- --nocapture`  
Expected: FAIL because `CameraCore` does not exist.

- [ ] **Step 3: Implement lifecycle state and use cases**

Use:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LifecycleState { Created, Running, Stopped }

pub struct CameraCore {
    ports: DevicePorts,
    lifecycle: tokio::sync::RwLock<LifecycleState>,
}
```

Operations retain a lifecycle read guard through the port call. `stop` takes the write guard, which drains readers, sets `Stopped`, and closes once. `start` opens only from `Created`, succeeds unchanged from `Running`, and returns `NotRunning` after `Stopped`.

- [ ] **Step 4: Run core-service tests and commit**

Run: `cargo test core::service::tests -- --nocapture && cargo clippy --lib -- -D warnings`  
Expected: PASS.  
Commit:

```bash
git add src/core
git commit -m "feat: add camera core lifecycle and use cases"
```

### Task 5: Simple fake device and infrastructure builder

**Files:**
- Create: `src/infrastructure/devices/mod.rs`
- Create: `src/infrastructure/devices/fake_simple.rs`
- Create: `src/infrastructure/builder.rs`
- Modify: `src/infrastructure/mod.rs`

**Interfaces:**
- Produces: `SimpleFakeCamera::new() -> SimpleFakeCamera`.
- Produces: `build_device(DeviceName) -> DevicePorts`.
- Implements: lifecycle, zoom, focus, and info ports from Task 3.

- [ ] **Step 1: Write failing fake-device tests**

Cover initial closed state, idempotent open/close, `DeviceUnavailable` while closed, zoom/focus initial value `0`, info text `Fake Simple Camera`, per-instance isolation, and concurrent writes that always leave a valid normalized value.

Builder test:

```rust
let ports = build_device(DeviceName::FakeSimple);
assert_eq!(ports.capabilities(), vec![Capability::Zoom, Capability::Focus, Capability::Info]);
assert!(ports.autofocus().is_none());
assert!(ports.stabilization().is_none());
```

- [ ] **Step 2: Run infrastructure tests to verify failure**

Run: `cargo test infrastructure:: -- --nocapture`  
Expected: FAIL because the fake and builder do not exist.

- [ ] **Step 3: Implement concurrency-safe per-instance state**

Use one `tokio::sync::RwLock<FakeState>`:

```rust
struct FakeState {
    open: bool,
    zoom: Zoom,
    focus: Focus,
}
```

Every capability operation checks `open` while holding the same guard used for the read or write. No lock guard may be reacquired recursively.

- [ ] **Step 4: Implement static typed device construction**

Construct one `Arc<SimpleFakeCamera>`, clone and coerce it into lifecycle, zoom, focus, and info trait objects, then finish `DevicePorts::builder`. Use an exhaustive match over `DeviceName`.

- [ ] **Step 5: Run infrastructure and cross-layer tests and commit**

Run: `cargo test infrastructure:: -- --nocapture && cargo test --lib && cargo clippy --lib -- -D warnings`  
Expected: PASS.  
Commit:

```bash
git add src/infrastructure
git commit -m "feat: add simple fake camera adapter"
```

### Task 6: Complete Tonic API adapter and error mapping

**Files:**
- Create: `src/api/grpc/status.rs`
- Create: `src/api/grpc/service.rs`
- Modify: `src/api/grpc/mod.rs`

**Interfaces:**
- Produces: `GrpcCameraService::new(Arc<CameraCore>)`.
- Implements: generated `proto::camera_service_server::CameraService`.
- Produces: centralized `impl From<DomainError> for tonic::Status`.

- [ ] **Step 1: Write failing status mapping tests**

Assert exact mappings:

```rust
assert_eq!(Status::from(invalid).code(), Code::InvalidArgument);
assert_eq!(Status::from(DomainError::UnsupportedCapability("autofocus")).code(), Code::Unimplemented);
assert_eq!(Status::from(DomainError::NotRunning).code(), Code::FailedPrecondition);
assert_eq!(Status::from(DomainError::DeviceUnavailable("closed".into())).code(), Code::Unavailable);
assert_eq!(Status::from(DomainError::Device("failed".into())).code(), Code::Internal);
```

- [ ] **Step 2: Run mapping tests to verify failure**

Run: `cargo test api::grpc::status::tests -- --nocapture`  
Expected: FAIL because mapping is absent.

- [ ] **Step 3: Implement centralized mapping and all RPC methods**

Each RPC converts only at the edge. Example:

```rust
async fn set_zoom(
    &self,
    request: Request<SetZoomRequest>,
) -> Result<Response<SetZoomResponse>, Status> {
    let zoom = Zoom::new(i64::from(request.into_inner().zoom)).map_err(Status::from)?;
    self.core.set_zoom(zoom).await.map_err(Status::from)?;
    Ok(Response::new(SetZoomResponse {}))
}
```

Implement all 12 generated trait methods. Convert capability identifiers to the unchanged protobuf enum. AutoFocus and Stabilization naturally map missing ports to `UNIMPLEMENTED`; do not special-case the fake in the API layer.

- [ ] **Step 4: Add per-RPC outcome and latency recording**

Use a shared helper that records method, `tonic::Code`, and elapsed milliseconds through `tracing`. Do not log full requests.

- [ ] **Step 5: Run API and core tests and commit**

Run: `cargo test api::grpc -- --nocapture && cargo test --lib && cargo clippy --lib -- -D warnings`  
Expected: PASS.  
Commit:

```bash
git add src/api
git commit -m "feat: implement camera gRPC API"
```

### Task 7: gRPC runtime, health, reflection, application, and CLI

**Files:**
- Create: `src/api/grpc/server.rs`
- Create: `src/app.rs`
- Modify: `src/main.rs`
- Modify: `src/api/grpc/mod.rs`
- Modify: `src/lib.rs`
- Create: `tests/application_lifecycle.rs`

**Interfaces:**
- Produces: `GrpcServer::new(Arc<CameraCore>)`.
- Produces: `GrpcServer::serve(TcpListener, CancellationToken) -> Result<(), GrpcServerError>`.
- Produces: `Application::from_config(Config)` and `run(CancellationToken)`.
- Produces: Clap `Cli { config: PathBuf }` with default `config/config.yaml`.

- [ ] **Step 1: Write failing server/application lifecycle tests**

Test with a loopback `TcpListener` bound to port `0`:

- Core starts before serving.
- Health becomes `SERVING`.
- Reflection lists `camera.v1.CameraService`.
- Cancelling the token terminates the server and closes the core.
- The listener address can be rebound after shutdown.

- [ ] **Step 2: Run lifecycle tests to verify failure**

Run: `cargo test application_lifecycle -- --nocapture`  
Expected: FAIL because server, app, and binary are absent.

- [ ] **Step 3: Implement Tonic runtime services**

Create health services with `tonic_health::server::health_reporter`, reflection from `FILE_DESCRIPTOR_SET`, and the generated `CameraServiceServer`. In the shutdown future, set the camera service to `NOT_SERVING` before returning to Tonic's graceful shutdown path.

Use `TcpListenerStream` and `serve_with_incoming_shutdown`; return errors instead of panicking.

- [ ] **Step 4: Implement composition and tracing**

`Application::from_config` exhaustively selects `DeviceName::FakeSimple` and `ApiType::Grpc`, builds `DevicePorts`, and creates `Arc<CameraCore>`. `run` starts core, binds `127.0.0.1:<port>`, serves until cancellation, and always attempts core shutdown before returning.

Initialize `tracing_subscriber` once using the configured level and contextual startup errors. Keep Tonic, YAML, and concrete device names out of core modules.

- [ ] **Step 5: Implement the thin binary and portable signal handling**

Use `#[tokio::main]`, `clap::Parser`, and a cancellation token. Wait for Ctrl+C everywhere and SIGTERM under `#[cfg(unix)]`, then cancel. Report the full error chain to standard error and exit nonzero on failure.

- [ ] **Step 6: Run lifecycle, binary-help, and lint checks; commit**

Run:

```bash
cargo test application_lifecycle -- --nocapture
cargo run -- --help
cargo run -- --version
cargo clippy --all-targets --all-features -- -D warnings
```

Expected: lifecycle PASS; help and version exit `0`; Clippy clean.  
Commit:

```bash
git add src
git commit -m "feat: compose runnable camera service"
```

### Task 8: Full gRPC integration, documentation, and CI

**Files:**
- Create: `tests/grpc_integration.rs`
- Modify: `tests/application_lifecycle.rs`
- Create: `README.md`
- Create: `.github/workflows/ci.yml`
- Modify: tests and source files only when a failing acceptance test exposes a requirement gap.

**Interfaces:**
- Consumes: generated client, `GrpcServer`, `CameraCore`, and `build_device`.
- Produces: documented developer commands and extension workflows.

- [ ] **Step 1: Write end-to-end tests for every RPC**

Start the real server on an ephemeral loopback listener, connect the generated client, and assert:

- Set/get zoom and focus round trips.
- Min/max zoom return `0` and `100` through subsequent reads.
- Values above `100` return `INVALID_ARGUMENT`.
- Info is exactly `Fake Simple Camera`.
- Capabilities are exactly Zoom, Focus, and Info.
- All four AutoFocus/Stabilization operations return `UNIMPLEMENTED`.
- Concurrent client tasks leave zoom and focus in `0..=100`.
- Cancellation shuts down the server cleanly.

- [ ] **Step 2: Run integration tests to verify failure**

Run: `cargo test --test grpc_integration -- --nocapture`  
Expected: at least one new acceptance test fails before any exposed gap is corrected.

- [ ] **Step 3: Make only the minimal implementation corrections required by acceptance tests**

Preserve layer direction and centralized status mapping. Do not add fake-device special cases in gRPC code.

- [ ] **Step 4: Document usage and extension points**

README must include prerequisites, `cargo run -- --config config/config.yaml`, `grpcurl`/`grpcui` discovery through reflection, all quality commands, the exact supported capabilities, and concise “Add a device,” “Add a capability,” and “Add an API adapter” instructions that match the design specification.

- [ ] **Step 5: Add GitHub Actions**

On pushes and pull requests, install Rust `1.96.1` with `rustfmt` and `clippy`, then run the four required gates. Cache Cargo registries and `target` using the official Rust/Cargo cache action.

- [ ] **Step 6: Run the full completion audit**

Run fresh:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
git diff --check
git status --short
```

Expected: every command exits `0`; all tests pass; only intended files are modified.

- [ ] **Step 7: Commit the verified service**

```bash
git add README.md .github tests src Cargo.toml Cargo.lock build.rs proto config
git commit -m "test: verify Rust camera service end to end"
```

- [ ] **Step 8: Publish for review**

Push the current `codex/` branch and open a pull request targeting `main`. Include the architecture summary, supported/unsupported capability behavior, approved YAML, and exact verification commands in the PR body.
