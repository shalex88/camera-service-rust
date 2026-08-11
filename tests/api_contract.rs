use std::collections::BTreeMap;

use camera_controller_rust::api::grpc::FILE_DESCRIPTOR_SET;
use prost::Message;
use prost_types::field_descriptor_proto::{Label, Type};
use prost_types::{DescriptorProto, EnumDescriptorProto, FileDescriptorSet};

type FieldSignature<'a> = (&'a str, i32, i32, i32, &'a str);

fn message_fields(messages: &[DescriptorProto]) -> BTreeMap<&str, Vec<FieldSignature<'_>>> {
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
                        field.label.unwrap_or_default(),
                        field.r#type.unwrap_or_default(),
                        field.type_name.as_deref().unwrap_or_default(),
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
    let method_signatures: Vec<_> = service
        .method
        .iter()
        .map(|method| {
            (
                method.name.as_deref().unwrap_or_default(),
                method.input_type.as_deref().unwrap_or_default(),
                method.output_type.as_deref().unwrap_or_default(),
                method.client_streaming.unwrap_or_default(),
                method.server_streaming.unwrap_or_default(),
            )
        })
        .collect();
    assert_eq!(
        method_signatures,
        [
            (
                "SetZoom",
                ".camera.v1.SetZoomRequest",
                ".camera.v1.SetZoomResponse",
                false,
                false,
            ),
            (
                "GetZoom",
                ".google.protobuf.Empty",
                ".camera.v1.GetZoomResponse",
                false,
                false,
            ),
            (
                "GoToMinZoom",
                ".google.protobuf.Empty",
                ".camera.v1.GoToMinZoomResponse",
                false,
                false,
            ),
            (
                "GoToMaxZoom",
                ".google.protobuf.Empty",
                ".camera.v1.GoToMaxZoomResponse",
                false,
                false,
            ),
            (
                "SetFocus",
                ".camera.v1.SetFocusRequest",
                ".camera.v1.SetFocusResponse",
                false,
                false,
            ),
            (
                "GetFocus",
                ".google.protobuf.Empty",
                ".camera.v1.GetFocusResponse",
                false,
                false,
            ),
            (
                "SetAutoFocus",
                ".camera.v1.SetAutoFocusRequest",
                ".google.protobuf.Empty",
                false,
                false,
            ),
            (
                "GetAutoFocus",
                ".google.protobuf.Empty",
                ".camera.v1.GetAutoFocusResponse",
                false,
                false,
            ),
            (
                "GetInfo",
                ".google.protobuf.Empty",
                ".camera.v1.GetInfoResponse",
                false,
                false,
            ),
            (
                "GetCapabilities",
                ".google.protobuf.Empty",
                ".camera.v1.GetCapabilitiesResponse",
                false,
                false,
            ),
            (
                "SetStabilization",
                ".camera.v1.SetStabilizationRequest",
                ".google.protobuf.Empty",
                false,
                false,
            ),
            (
                "GetStabilization",
                ".google.protobuf.Empty",
                ".camera.v1.GetStabilizationResponse",
                false,
                false,
            ),
        ]
    );

    let fields = message_fields(&file.message_type);
    let optional = Label::Optional as i32;
    let repeated = Label::Repeated as i32;
    let uint32 = Type::Uint32 as i32;
    let boolean = Type::Bool as i32;
    let string = Type::String as i32;
    let enumeration = Type::Enum as i32;
    let expected_fields = BTreeMap::from([
        (
            "GetAutoFocusResponse",
            vec![("enable", 1, optional, boolean, "")],
        ),
        (
            "GetCapabilitiesResponse",
            vec![(
                "capabilities",
                1,
                repeated,
                enumeration,
                ".camera.v1.Capability",
            )],
        ),
        ("GetFocusResponse", vec![("focus", 1, optional, uint32, "")]),
        ("GetInfoResponse", vec![("info", 1, optional, string, "")]),
        (
            "GetStabilizationResponse",
            vec![("enable", 1, optional, boolean, "")],
        ),
        ("GetZoomResponse", vec![("zoom", 1, optional, uint32, "")]),
        ("GoToMaxZoomResponse", vec![]),
        ("GoToMinZoomResponse", vec![]),
        (
            "SetAutoFocusRequest",
            vec![("enable", 1, optional, boolean, "")],
        ),
        ("SetFocusRequest", vec![("focus", 1, optional, uint32, "")]),
        ("SetFocusResponse", vec![]),
        (
            "SetStabilizationRequest",
            vec![("enable", 1, optional, boolean, "")],
        ),
        ("SetZoomRequest", vec![("zoom", 1, optional, uint32, "")]),
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
