use std::collections::BTreeMap;

use camera_controller_rust::api::grpc::FILE_DESCRIPTOR_SET;
use prost::Message;
use prost_types::{DescriptorProto, EnumDescriptorProto, FileDescriptorSet};

fn message_fields(messages: &[DescriptorProto]) -> BTreeMap<&str, Vec<(&str, i32)>> {
    messages
        .iter()
        .map(|message| {
            let fields = message
                .field
                .iter()
                .map(|field| {
                    (
                        field.name.as_deref().unwrap_or_default(),
                        field.number.unwrap_or_default(),
                    )
                })
                .collect();
            (message.name.as_deref().unwrap_or_default(), fields)
        })
        .collect()
}

fn enum_values(enumeration: &EnumDescriptorProto) -> Vec<(&str, i32)> {
    enumeration
        .value
        .iter()
        .map(|value| {
            (
                value.name.as_deref().unwrap_or_default(),
                value.number.unwrap_or_default(),
            )
        })
        .collect()
}

#[test]
fn generated_descriptor_preserves_camera_v1_contract() {
    let descriptor = FileDescriptorSet::decode(FILE_DESCRIPTOR_SET)
        .expect("generated descriptor must be decodable");
    let file = descriptor
        .file
        .iter()
        .find(|file| file.package.as_deref() == Some("camera.v1"))
        .expect("camera.v1 file must exist");
    let service = file
        .service
        .iter()
        .find(|service| service.name.as_deref() == Some("CameraService"))
        .expect("CameraService must exist");

    let method_names: Vec<_> = service
        .method
        .iter()
        .map(|method| method.name.as_deref().unwrap_or_default())
        .collect();
    assert_eq!(
        method_names,
        [
            "SetZoom",
            "GetZoom",
            "GoToMinZoom",
            "GoToMaxZoom",
            "SetFocus",
            "GetFocus",
            "SetAutoFocus",
            "GetAutoFocus",
            "GetInfo",
            "GetCapabilities",
            "SetStabilization",
            "GetStabilization",
        ]
    );

    let fields = message_fields(&file.message_type);
    let expected_fields = BTreeMap::from([
        ("GetAutoFocusResponse", vec![("enable", 1)]),
        ("GetCapabilitiesResponse", vec![("capabilities", 1)]),
        ("GetFocusResponse", vec![("focus", 1)]),
        ("GetInfoResponse", vec![("info", 1)]),
        ("GetStabilizationResponse", vec![("enable", 1)]),
        ("GetZoomResponse", vec![("zoom", 1)]),
        ("GoToMaxZoomResponse", vec![]),
        ("GoToMinZoomResponse", vec![]),
        ("SetAutoFocusRequest", vec![("enable", 1)]),
        ("SetFocusRequest", vec![("focus", 1)]),
        ("SetFocusResponse", vec![]),
        ("SetStabilizationRequest", vec![("enable", 1)]),
        ("SetZoomRequest", vec![("zoom", 1)]),
        ("SetZoomResponse", vec![]),
    ]);
    assert_eq!(fields, expected_fields);

    let capability = file
        .enum_type
        .iter()
        .find(|enumeration| enumeration.name.as_deref() == Some("Capability"))
        .expect("Capability enum must exist");
    assert_eq!(
        enum_values(capability),
        [
            ("CAPABILITY_UNSPECIFIED", 0),
            ("CAPABILITY_ZOOM", 1),
            ("CAPABILITY_FOCUS", 2),
            ("CAPABILITY_AUTO_FOCUS", 3),
            ("CAPABILITY_INFO", 4),
            ("CAPABILITY_STABILIZATION", 5),
        ]
    );
}
