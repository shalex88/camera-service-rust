mod server;
mod service;
mod status;

pub use server::{GrpcServer, GrpcServerError};
pub use service::GrpcCameraService;

pub mod proto {
    tonic::include_proto!("camera.v1");
}

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("camera_descriptor");
