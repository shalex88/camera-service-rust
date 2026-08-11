# Rust Camera Service Design

**Status:** Approved for implementation planning  
**Date:** 2026-08-11  
**Reference:** [`shalex88/camera-controller` branch `feature/mwir`](https://github.com/shalex88/camera-controller/tree/feature/mwir), inspected at commit `b21212d`

## 1. Purpose

Build an idiomatic Rust replacement for the reference camera-controller service. The first release runs on a developer machine and uses only an in-memory simple fake camera. Its primary architectural goal is to make new devices, device capabilities, and API adapters straightforward to add without coupling them to each other.

The service preserves the complete `camera.v1.CameraService` gRPC wire contract from the reference repository while replacing the C++ inheritance hierarchy with explicit Rust ports, typed values, ownership, and dependency injection.

## 2. Scope

### Included

- One Cargo package with a reusable library target and a thin binary target, using Rust edition 2024 and minimum Rust version 1.96.
- Three explicit layers: API, core, and infrastructure.
- The complete existing `camera.v1.CameraService` protobuf contract.
- A simple fake camera with zoom, focus, and device-information capabilities.
- Typed YAML configuration loaded with `--config`.
- Structured logging, gRPC health, reflection, and graceful shutdown.
- Unit, configuration, contract, integration, and application-lifecycle tests.
- Developer-oriented GitHub Actions checks matching the required local checks.

### Excluded from the first release

- MWIR, Sony, Adimec, or other physical camera implementations.
- ITL, VISCA, GenICam, TCP, UART, UIO, MMIO, or FPGA support.
- Cross-compilation, embedded deployment, containers, systemd, or native packages.
- Persistence across process restarts.
- Authentication, TLS, or exposure beyond the local developer machine.
- Runtime-loaded dynamic plugins.
- REST, WebSocket, or other non-gRPC API adapters.

These exclusions are adapter work, not architectural limitations. The core boundaries are designed so they can be added without changing existing device implementations.

## 3. Architectural Principles

1. Dependencies point inward toward the core.
2. Protobuf, Tonic, YAML, and concrete device types never appear in core interfaces.
3. Devices implement small capability ports rather than one all-purpose camera trait.
4. Unsupported behavior is represented explicitly, not by dummy implementations.
5. Invalid domain state is unrepresentable through validated newtypes.
6. The composition root is the only place that knows concrete API and device adapters.
7. No `unsafe` code, global mutable state, downcasting, `Any`, service locators, or inheritance emulation.
8. Expected failures use typed errors. Panics are reserved for defects and are not used for input, configuration, lifecycle, or device failures.

## 4. Three-Layer Architecture

```text
API ─────────────► Core ◄──────────── Infrastructure
                        ▲
                        │ constructs and owns lifecycle
                     App/Main
```

### 4.1 API layer

The API layer owns transport-specific concerns:

- Tonic server construction and shutdown.
- Protobuf-generated request and response types.
- Conversion between protobuf values and core domain values.
- Conversion from core errors to gRPC status codes.
- gRPC health and reflection services.
- RPC tracing fields and latency measurement.

It depends only on the public core service interface. It never imports `SimpleFakeCamera` or any other infrastructure type.

### 4.2 Core layer

The core owns application behavior and stable extension points:

- Validated `Zoom` and `Focus` values.
- Capability identifiers.
- Capability-specific device ports.
- Device lifecycle coordination.
- Capability discovery derived from registered ports.
- Use cases for every method in `camera.v1.CameraService`.
- Typed domain errors.

The core has no dependency on the API or infrastructure layers.

### 4.3 Infrastructure layer

The infrastructure layer implements core ports with concrete adapters. The first adapter is `SimpleFakeCamera`, which stores per-instance in-memory state and implements:

- `DeviceLifecycle`
- `ZoomCapability`
- `FocusCapability`
- `InfoCapability`

It does not implement autofocus or stabilization. Missing capability ports are the authoritative representation of unsupported behavior.

### 4.4 Composition root

`app.rs` loads validated configuration, initializes tracing, builds the selected device ports, creates the core service, constructs the selected API runtime, and coordinates startup and graceful shutdown. It contains no request or device business rules.

## 5. Proposed Source Structure

```text
Cargo.toml
Cargo.lock
build.rs
proto/
└── camera_service.proto
src/
├── lib.rs
├── api/
│   ├── mod.rs
│   └── grpc/
│       ├── mod.rs
│       ├── server.rs
│       ├── service.rs
│       └── status.rs
├── core/
│   ├── mod.rs
│   ├── capabilities/
│   │   ├── mod.rs
│   │   ├── autofocus.rs
│   │   ├── focus.rs
│   │   ├── info.rs
│   │   ├── stabilization.rs
│   │   └── zoom.rs
│   ├── device.rs
│   ├── error.rs
│   └── service.rs
├── infrastructure/
│   ├── mod.rs
│   ├── builder.rs
│   └── devices/
│       ├── mod.rs
│       └── fake_simple.rs
├── app.rs
├── config.rs
└── main.rs
config/
└── config.yaml
tests/
├── api_contract.rs
├── grpc_integration.rs
└── application_lifecycle.rs
```

Each file has one principal responsibility. `lib.rs` exposes the modules needed by integration tests and keeps `main.rs` limited to argument parsing, application invocation, and exit reporting. Generated Rust protobuf files remain in Cargo's build output and are included by the API module; they are not committed.

## 6. Core Model and Extension Ports

### 6.1 Validated domain values

`Zoom` and `Focus` are newtypes over `u8`. Their public constructors accept integer inputs and return `DomainError::InvalidArgument` unless the value is in `0..=100`. Their inner values are readable through explicit accessors. No public API allows an out-of-range instance to be constructed.

The fake camera stores normalized domain values directly. It does not reproduce the C++ fake's internal `0..255` hardware range because no hardware conversion exists in this adapter.

### 6.2 Capability ports

The core defines object-safe, asynchronous, `Send + Sync` traits:

```rust
#[async_trait::async_trait]
pub trait DeviceLifecycle: Send + Sync {
    async fn open(&self) -> Result<(), DomainError>;
    async fn close(&self) -> Result<(), DomainError>;
}

#[async_trait::async_trait]
pub trait ZoomCapability: Send + Sync {
    async fn set_zoom(&self, zoom: Zoom) -> Result<(), DomainError>;
    async fn zoom(&self) -> Result<Zoom, DomainError>;
}

#[async_trait::async_trait]
pub trait FocusCapability: Send + Sync {
    async fn set_focus(&self, focus: Focus) -> Result<(), DomainError>;
    async fn focus(&self) -> Result<Focus, DomainError>;
}

#[async_trait::async_trait]
pub trait InfoCapability: Send + Sync {
    async fn info(&self) -> Result<DeviceInfo, DomainError>;
}
```

Autofocus and stabilization use equally focused traits in their own modules. The async boundary is intentional even though the fake is in-memory: future real adapters can perform nonblocking I/O without changing the core or API contracts.

### 6.3 Device port aggregation

`DevicePorts` contains one required lifecycle port and optional capability ports:

```rust
pub struct DevicePorts {
    lifecycle: Arc<dyn DeviceLifecycle>,
    zoom: Option<Arc<dyn ZoomCapability>>,
    focus: Option<Arc<dyn FocusCapability>>,
    info: Option<Arc<dyn InfoCapability>>,
    autofocus: Option<Arc<dyn AutoFocusCapability>>,
    stabilization: Option<Arc<dyn StabilizationCapability>>,
}
```

A typed builder constructs this aggregate. The same `Arc<SimpleFakeCamera>` is coerced into each trait object it implements, so all capabilities share one device instance and one lifecycle.

The core derives its capability response from fields that are present in `DevicePorts`. Adapters do not maintain a second capability list, preventing reported support from disagreeing with callable behavior.

### 6.4 Core service

The core service owns `DevicePorts` and a synchronized lifecycle state. It exposes use-case methods corresponding to the gRPC contract but returns only core types.

Lifecycle state is:

```text
Created → Running → Stopped
```

Startup and shutdown are idempotent. Active operations hold a lifecycle read guard; shutdown takes the write guard, waits for active operations to finish, marks the service stopped, and closes the device. This prevents a request from racing with device closure.

For each operation, the core:

1. Verifies that the service is running.
2. Validates and constructs domain inputs.
3. Looks up the required capability port.
4. Returns `UnsupportedCapability` when the port is absent.
5. Invokes the port and returns its typed result.

## 7. Simple Fake Device Behavior

The initial state for every new instance is:

| Property | Value |
|---|---:|
| Open | `false` |
| Zoom | `0` |
| Focus | `0` |
| Info | `"Fake Simple Camera"` |

State belongs to the instance and is protected by `tokio::sync::RwLock`. Two fake-camera instances never share state. Reads may proceed concurrently; writes are exclusive.

`open` and `close` are idempotent. Capability methods defensively reject calls while closed with `DomainError::DeviceUnavailable`, even though the core lifecycle normally prevents such calls.

The adapter supports zoom, focus, and info. It has no autofocus or stabilization implementations.

## 8. gRPC Contract and Behavior

`proto/camera_service.proto` is copied unchanged from the reference branch. The following remain unchanged:

- Package `camera.v1`.
- Service and RPC names.
- Request and response message names.
- Field names, types, and numbers.
- Capability enum names and numeric values.

The full API remains available:

| RPC | First-release behavior |
|---|---|
| `SetZoom` | Validate and store `0..=100` |
| `GetZoom` | Return stored zoom |
| `GoToMinZoom` | Store `0` |
| `GoToMaxZoom` | Store `100` |
| `SetFocus` | Validate and store `0..=100` |
| `GetFocus` | Return stored focus |
| `SetAutoFocus` | Return `UNIMPLEMENTED` |
| `GetAutoFocus` | Return `UNIMPLEMENTED` |
| `GetInfo` | Return `"Fake Simple Camera"` |
| `GetCapabilities` | Return Zoom, Focus, and Info exactly once each |
| `SetStabilization` | Return `UNIMPLEMENTED` |
| `GetStabilization` | Return `UNIMPLEMENTED` |

Tonic and Prost code generation occurs in `build.rs`. A vendored `protoc` binary makes developer and CI builds reproducible without a system protobuf installation.

The server binds to `127.0.0.1` using the configured port. Transport security is intentionally excluded because the first release is loopback-only. A future non-loopback configuration must add authentication and TLS requirements before it is enabled.

Standard gRPC health and server-reflection services are enabled. Health reports `SERVING` only after core startup completes and changes to `NOT_SERVING` before graceful shutdown begins.

## 9. Configuration Contract

The required YAML shape is:

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

Typed configuration uses `serde` with unknown-field rejection. Enum-backed fields accept only:

- `api_type: grpc`
- `device_type: camera`
- `device_name: fake_simple`
- `log_level`: `trace`, `debug`, `info`, `warn`, or `error`

The port must be in `1..=65535`, and the application name must be nonempty after trimming. Missing fields, unknown fields, invalid enum values, invalid ports, unreadable files, and malformed YAML produce startup errors with field-path context.

`clap` provides:

- `-c, --config <FILE>` with default `config/config.yaml`.
- `-V, --version` using Cargo package metadata.
- Generated `-h, --help` output.

Configuration is loaded and validated before tracing, devices, or network listeners are initialized. Errors are printed once to standard error and cause a nonzero exit.

## 10. Error Model

Reusable modules use concrete errors derived with `thiserror`. `anyhow` is permitted only in the binary/composition boundary to attach startup and shutdown context.

Core errors map to gRPC status codes as follows:

| Core error | gRPC status |
|---|---|
| `InvalidArgument` | `INVALID_ARGUMENT` |
| `UnsupportedCapability` | `UNIMPLEMENTED` |
| `NotRunning` | `FAILED_PRECONDITION` |
| `DeviceUnavailable` | `UNAVAILABLE` |
| `Device` | `INTERNAL` |

Error messages identify the failed operation without exposing implementation details. The API mapping is centralized in `api/grpc/status.rs`; individual RPC methods do not define ad hoc mappings.

## 11. Concurrency and Shutdown

Tonic handles requests concurrently on Tokio. Shared values use `Arc`; mutable fake state uses Tokio synchronization primitives. No standard blocking mutex is held across an `.await` point.

The process reacts to Ctrl+C on every developer platform and SIGTERM on Unix. A cancellation token coordinates API shutdown. The shutdown sequence is:

1. Mark gRPC health `NOT_SERVING`.
2. Signal Tonic graceful shutdown so no new calls are accepted.
3. Allow active RPCs to complete.
4. Stop the core and close the fake device.
5. Flush tracing output and return success.

Startup failures unwind already-created components in reverse ownership order. Repeated shutdown requests and repeated close calls are harmless.

## 12. Observability

`tracing` and `tracing-subscriber` provide structured logs controlled by `app.log_level`. Startup logs contain application name, version, configuration path, and loopback listening address.

Each RPC span records:

- gRPC method.
- Result status code.
- Elapsed time.

Request payloads are not logged wholesale. Routine invalid input and unsupported capabilities do not emit stack traces. Unexpected adapter failures include their error chain at the composition boundary.

## 13. Extending the Service

### Add a device

1. Add one module beneath `infrastructure/devices/`.
2. Implement `DeviceLifecycle` and only the capability traits the device supports.
3. Add a typed `DeviceName` configuration variant.
4. Add one exhaustive construction arm in `infrastructure/builder.rs`.
5. Register supported ports in `DevicePorts` and add adapter tests.

The core and existing API adapters do not change.

### Add a capability

1. Add a focused module beneath `core/capabilities/` containing its domain types and port trait.
2. Add its optional port and capability identifier to `DevicePorts`.
3. Add core use cases and error tests.
4. Extend an existing protobuf API compatibly or introduce a new version for breaking changes.
5. Map the API operation to the new core use case.
6. Implement the trait only for devices that support it.

Existing devices compile without dummy behavior and report the capability as unsupported until they opt in.

### Add an API adapter

1. Add a module beneath `api/` implementing the API runtime lifecycle.
2. Translate transport inputs into existing core types and errors into transport responses.
3. Add a typed `ApiType` configuration variant and composition arm.
4. Add adapter-level integration tests.

Device adapters do not change. A future configuration revision may support multiple simultaneous API runtimes; the first release intentionally starts one configured runtime.

## 14. Testing Strategy

### Core unit tests

- Accept `0` and `100` for zoom and focus.
- Reject values below `0` or above `100` at domain construction.
- Enforce running lifecycle state.
- Derive capabilities from present ports.
- Route each supported operation to its port.
- Return `UnsupportedCapability` for absent ports.
- Set exact min/max zoom values.
- Drain active operations before shutdown.

### Infrastructure unit tests

- Verify initial values and info text.
- Verify per-instance state isolation.
- Verify concurrent read/write safety.
- Verify open and close idempotency.
- Reject capability calls while closed.
- Verify only zoom, focus, and info ports are registered.

### Configuration tests

- Parse the exact approved YAML.
- Reject missing and unknown fields.
- Reject unsupported API, device type, and device name values.
- Reject empty names and invalid ports.
- Report useful path context for nested failures.

### Contract and API integration tests

- Verify protobuf package, RPC names, message field numbers, and capability enum values.
- Start a real server on an ephemeral loopback port and use the generated Tonic client.
- Exercise every RPC and verify successful responses or exact gRPC status codes.
- Verify capabilities contain Zoom, Focus, and Info exactly once.
- Verify health transitions and reflection availability.
- Verify concurrent requests leave valid state.

### Application tests

- Verify invalid configuration fails before binding.
- Verify startup, request handling, and graceful shutdown.
- Verify the configured port is released after shutdown.

## 15. Quality Gates

Every local handoff and CI run must pass:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

The crate root denies unsafe code. `Cargo.lock` is committed. Public core extension traits and configuration fields have concise Rustdoc. CI runs on pull requests and branch pushes with Rust 1.96.1.

## 16. Migration Sequence

1. Preserve the protobuf and YAML contracts as executable tests.
2. Scaffold the Rust crate, build-time protobuf generation, lint policy, and CI checks.
3. Implement validated core values, typed errors, lifecycle, and capability ports test-first.
4. Implement and register `SimpleFakeCamera` test-first.
5. Implement the Tonic API adapter and centralized status mapping test-first.
6. Add typed configuration, composition, health, reflection, tracing, and graceful shutdown.
7. Run contract and full-stack integration tests against every RPC.
8. Document developer usage and the device, capability, and API extension workflows.

Each sequence item must leave the project compiling and its relevant tests passing. Detailed file-level tasks and commit boundaries belong in the implementation plan derived from this specification.

## 17. Acceptance Criteria

The first migration milestone is complete when:

- The approved YAML starts the service on `127.0.0.1:50051`.
- A generated `camera.v1.CameraService` client can call all existing RPCs.
- Zoom, focus, min/max zoom, info, and capability discovery behave as specified.
- Autofocus and stabilization consistently return `UNIMPLEMENTED`.
- Invalid zoom and focus consistently return `INVALID_ARGUMENT`.
- Health, reflection, structured logging, and graceful shutdown work.
- All required quality gates pass on the developer machine and in CI.
- The documented extension workflows do not require an existing device adapter to implement unsupported capabilities.
