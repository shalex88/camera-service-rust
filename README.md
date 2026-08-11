# Camera Controller Rust

An extensible, developer-focused camera control service written in idiomatic Rust. It preserves the complete `camera.v1.CameraService` gRPC contract from the C++ camera controller while using explicit three-layer boundaries and capability-specific device ports.

The first release uses an in-memory `fake_simple` camera. It has no physical-camera, network-device, UART, MMIO, or FPGA dependencies.

## Architecture

```text
gRPC API ─────────► Core ◄───────── Infrastructure
                     ▲                SimpleFakeCamera
                     │
                  App/Main
```

- `api` owns Tonic, protobuf conversion, status mapping, health, and reflection.
- `core` owns validated values, use cases, lifecycle, errors, and capability ports.
- `infrastructure` implements core ports with concrete devices.
- `app` is the composition root and owns startup and shutdown order.

Neither API adapters nor device adapters depend on each other.

## Requirements

- Rust 1.96.1 or newer.
- A developer machine that can bind a loopback TCP port.
- Optional: [`grpcurl`](https://github.com/fullstorydev/grpcurl) or [`grpcui`](https://github.com/fullstorydev/grpcui) for interactive calls.

The build uses a vendored `protoc`; a system protobuf compiler is not required.

## Run

```bash
cargo run -- --config config/config.yaml
```

The default configuration listens on `127.0.0.1:50051`:

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

Unknown, missing, or invalid YAML fields stop startup with a path-aware error.

Discover the API through reflection:

```bash
grpcurl -plaintext 127.0.0.1:50051 list
grpcurl -plaintext 127.0.0.1:50051 describe camera.v1.CameraService
grpcui -plaintext 127.0.0.1:50051
```

Stop the service with Ctrl+C. Unix SIGTERM is also handled gracefully.

## Current device behavior

`fake_simple` stores state in memory and resets on restart.

| Capability | Behavior |
|---|---|
| Zoom | Set/get normalized values `0..=100` |
| Focus | Set/get normalized values `0..=100` |
| Info | Returns `Fake Simple Camera` |
| AutoFocus | gRPC `UNIMPLEMENTED` |
| Stabilization | gRPC `UNIMPLEMENTED` |

`GetCapabilities` returns Zoom, Focus, and Info. Invalid normalized values return `INVALID_ARGUMENT`.

## Quality checks

Run the same gates used by CI:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

## Add a device

1. Add a module under `src/infrastructure/devices/`.
2. Implement `DeviceLifecycle` and only the core capability traits the device supports.
3. Add a `DeviceName` variant in `src/config.rs`.
4. Add one exhaustive construction arm in `src/infrastructure/builder.rs`.
5. Register only the implemented capability ports and add adapter tests.

The core and gRPC adapter do not change.

## Add a capability

1. Add a focused domain type and async port under `src/core/capabilities/`.
2. Add an optional field and builder method to `DevicePorts`.
3. Add the core use case and its tests.
4. Extend the protobuf compatibly, or introduce `camera.v2` for a breaking contract.
5. Map the transport operation to the core use case.
6. Implement the port only for devices that support it.

Existing devices remain unsupported by default and need no dummy implementation.

## Add an API adapter

1. Add an adapter module under `src/api/`.
2. Translate transport inputs into core types and map `DomainError` into transport errors.
3. Add an `ApiType` variant and composition arm in `src/app.rs`.
4. Add real adapter-level integration tests.

Device adapters do not change. API adapters call the core and never concrete devices.

## Design documents

- [Approved design](docs/superpowers/specs/2026-08-11-rust-camera-service-design.md)
- [Implementation plan](docs/superpowers/plans/2026-08-11-rust-camera-service.md)
