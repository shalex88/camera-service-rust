use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    let protobuf_include = protoc_bin_vendored::include_path()?;
    let descriptor_path = PathBuf::from(std::env::var("OUT_DIR")?).join("camera_descriptor.bin");

    let mut prost_config = prost_build::Config::new();
    prost_config.protoc_executable(protoc);

    tonic_prost_build::configure()
        .file_descriptor_set_path(descriptor_path)
        .compile_with_config(
            prost_config,
            &[PathBuf::from("proto/camera_service.proto")],
            &[PathBuf::from("proto"), protobuf_include],
        )?;

    println!("cargo:rerun-if-changed=proto/camera_service.proto");
    Ok(())
}
