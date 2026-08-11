use std::sync::Arc;
use std::time::Instant;

use tonic::{Code, Request, Response, Status};

use crate::api::grpc::proto::camera_service_server::CameraService;
use crate::api::grpc::proto::{
    Capability as ProtoCapability, GetAutoFocusResponse, GetCapabilitiesResponse, GetFocusResponse,
    GetInfoResponse, GetStabilizationResponse, GetZoomResponse, GoToMaxZoomResponse,
    GoToMinZoomResponse, SetAutoFocusRequest, SetFocusRequest, SetFocusResponse,
    SetStabilizationRequest, SetZoomRequest, SetZoomResponse,
};
use crate::core::{CameraCore, Capability, Focus, Zoom};

/// The gRPC transport adapter for camera core use cases.
#[derive(Clone)]
pub struct GrpcCameraService {
    core: Arc<CameraCore>,
}

impl GrpcCameraService {
    /// Creates a transport adapter around a running camera core.
    pub fn new(core: Arc<CameraCore>) -> Self {
        Self { core }
    }
}

fn finish_rpc<T>(
    method: &'static str,
    started: Instant,
    result: Result<Response<T>, Status>,
) -> Result<Response<T>, Status> {
    let status = result
        .as_ref()
        .map_or_else(|error| error.code(), |_| Code::Ok);
    tracing::info!(
        rpc.method = method,
        grpc.status = ?status,
        elapsed_ms = started.elapsed().as_millis(),
        "camera RPC completed"
    );
    result
}

const fn to_proto_capability(capability: Capability) -> ProtoCapability {
    match capability {
        Capability::Zoom => ProtoCapability::Zoom,
        Capability::Focus => ProtoCapability::Focus,
        Capability::AutoFocus => ProtoCapability::AutoFocus,
        Capability::Info => ProtoCapability::Info,
        Capability::Stabilization => ProtoCapability::Stabilization,
    }
}

#[tonic::async_trait]
impl CameraService for GrpcCameraService {
    async fn set_zoom(
        &self,
        request: Request<SetZoomRequest>,
    ) -> Result<Response<SetZoomResponse>, Status> {
        let started = Instant::now();
        let result = async {
            let zoom = Zoom::new(i64::from(request.into_inner().zoom)).map_err(Status::from)?;
            self.core.set_zoom(zoom).await.map_err(Status::from)?;
            Ok(Response::new(SetZoomResponse {}))
        }
        .await;
        finish_rpc("SetZoom", started, result)
    }

    async fn get_zoom(&self, _request: Request<()>) -> Result<Response<GetZoomResponse>, Status> {
        let started = Instant::now();
        let result = self
            .core
            .zoom()
            .await
            .map(|zoom| {
                Response::new(GetZoomResponse {
                    zoom: u32::from(zoom.value()),
                })
            })
            .map_err(Status::from);
        finish_rpc("GetZoom", started, result)
    }

    async fn go_to_min_zoom(
        &self,
        _request: Request<()>,
    ) -> Result<Response<GoToMinZoomResponse>, Status> {
        let started = Instant::now();
        let result = self
            .core
            .go_to_min_zoom()
            .await
            .map(|()| Response::new(GoToMinZoomResponse {}))
            .map_err(Status::from);
        finish_rpc("GoToMinZoom", started, result)
    }

    async fn go_to_max_zoom(
        &self,
        _request: Request<()>,
    ) -> Result<Response<GoToMaxZoomResponse>, Status> {
        let started = Instant::now();
        let result = self
            .core
            .go_to_max_zoom()
            .await
            .map(|()| Response::new(GoToMaxZoomResponse {}))
            .map_err(Status::from);
        finish_rpc("GoToMaxZoom", started, result)
    }

    async fn set_focus(
        &self,
        request: Request<SetFocusRequest>,
    ) -> Result<Response<SetFocusResponse>, Status> {
        let started = Instant::now();
        let result = async {
            let focus = Focus::new(i64::from(request.into_inner().focus)).map_err(Status::from)?;
            self.core.set_focus(focus).await.map_err(Status::from)?;
            Ok(Response::new(SetFocusResponse {}))
        }
        .await;
        finish_rpc("SetFocus", started, result)
    }

    async fn get_focus(&self, _request: Request<()>) -> Result<Response<GetFocusResponse>, Status> {
        let started = Instant::now();
        let result = self
            .core
            .focus()
            .await
            .map(|focus| {
                Response::new(GetFocusResponse {
                    focus: u32::from(focus.value()),
                })
            })
            .map_err(Status::from);
        finish_rpc("GetFocus", started, result)
    }

    async fn set_auto_focus(
        &self,
        request: Request<SetAutoFocusRequest>,
    ) -> Result<Response<()>, Status> {
        let started = Instant::now();
        let result = self
            .core
            .set_auto_focus(request.into_inner().enable)
            .await
            .map(|()| Response::new(()))
            .map_err(Status::from);
        finish_rpc("SetAutoFocus", started, result)
    }

    async fn get_auto_focus(
        &self,
        _request: Request<()>,
    ) -> Result<Response<GetAutoFocusResponse>, Status> {
        let started = Instant::now();
        let result = self
            .core
            .auto_focus_enabled()
            .await
            .map(|enable| Response::new(GetAutoFocusResponse { enable }))
            .map_err(Status::from);
        finish_rpc("GetAutoFocus", started, result)
    }

    async fn get_info(&self, _request: Request<()>) -> Result<Response<GetInfoResponse>, Status> {
        let started = Instant::now();
        let result = self
            .core
            .info()
            .await
            .map(|info| {
                Response::new(GetInfoResponse {
                    info: info.as_str().to_owned(),
                })
            })
            .map_err(Status::from);
        finish_rpc("GetInfo", started, result)
    }

    async fn get_capabilities(
        &self,
        _request: Request<()>,
    ) -> Result<Response<GetCapabilitiesResponse>, Status> {
        let started = Instant::now();
        let result = self
            .core
            .capabilities()
            .await
            .map(|capabilities| GetCapabilitiesResponse {
                capabilities: capabilities
                    .into_iter()
                    .map(|capability| to_proto_capability(capability) as i32)
                    .collect(),
            })
            .map(Response::new)
            .map_err(Status::from);
        finish_rpc("GetCapabilities", started, result)
    }

    async fn set_stabilization(
        &self,
        request: Request<SetStabilizationRequest>,
    ) -> Result<Response<()>, Status> {
        let started = Instant::now();
        let result = self
            .core
            .set_stabilization(request.into_inner().enable)
            .await
            .map(|()| Response::new(()))
            .map_err(Status::from);
        finish_rpc("SetStabilization", started, result)
    }

    async fn get_stabilization(
        &self,
        _request: Request<()>,
    ) -> Result<Response<GetStabilizationResponse>, Status> {
        let started = Instant::now();
        let result = self
            .core
            .stabilization_enabled()
            .await
            .map(|enable| Response::new(GetStabilizationResponse { enable }))
            .map_err(Status::from);
        finish_rpc("GetStabilization", started, result)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tonic::{Code, Request};

    use super::GrpcCameraService;
    use crate::api::grpc::proto::camera_service_server::CameraService;
    use crate::api::grpc::proto::{
        SetAutoFocusRequest, SetFocusRequest, SetStabilizationRequest, SetZoomRequest,
    };
    use crate::config::DeviceName;
    use crate::core::CameraCore;
    use crate::infrastructure::build_device;

    async fn running_service() -> GrpcCameraService {
        let core = Arc::new(CameraCore::new(build_device(DeviceName::FakeSimple)));
        core.start().await.expect("core must start");
        GrpcCameraService::new(core)
    }

    #[tokio::test]
    async fn exposes_all_supported_simple_fake_operations() {
        let service = running_service().await;

        service
            .set_zoom(Request::new(SetZoomRequest { zoom: 44 }))
            .await
            .expect("set zoom must succeed");
        assert_eq!(
            service
                .get_zoom(Request::new(()))
                .await
                .expect("get zoom must succeed")
                .into_inner()
                .zoom,
            44
        );

        service
            .go_to_max_zoom(Request::new(()))
            .await
            .expect("max zoom must succeed");
        assert_eq!(
            service
                .get_zoom(Request::new(()))
                .await
                .expect("get max zoom must succeed")
                .into_inner()
                .zoom,
            100
        );
        service
            .go_to_min_zoom(Request::new(()))
            .await
            .expect("min zoom must succeed");
        assert_eq!(
            service
                .get_zoom(Request::new(()))
                .await
                .expect("get min zoom must succeed")
                .into_inner()
                .zoom,
            0
        );

        service
            .set_focus(Request::new(SetFocusRequest { focus: 36 }))
            .await
            .expect("set focus must succeed");
        assert_eq!(
            service
                .get_focus(Request::new(()))
                .await
                .expect("get focus must succeed")
                .into_inner()
                .focus,
            36
        );
        assert_eq!(
            service
                .get_info(Request::new(()))
                .await
                .expect("get info must succeed")
                .into_inner()
                .info,
            "Fake Simple Camera"
        );
        assert_eq!(
            service
                .get_capabilities(Request::new(()))
                .await
                .expect("capabilities must succeed")
                .into_inner()
                .capabilities,
            [1, 2, 4]
        );
    }

    #[tokio::test]
    async fn maps_invalid_normalized_values_to_invalid_argument() {
        let service = running_service().await;

        let zoom_error = service
            .set_zoom(Request::new(SetZoomRequest { zoom: 101 }))
            .await
            .expect_err("invalid zoom must fail");
        let focus_error = service
            .set_focus(Request::new(SetFocusRequest { focus: 101 }))
            .await
            .expect_err("invalid focus must fail");

        assert_eq!(zoom_error.code(), Code::InvalidArgument);
        assert_eq!(focus_error.code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn maps_every_missing_simple_fake_capability_to_unimplemented() {
        let service = running_service().await;

        let errors = [
            service
                .set_auto_focus(Request::new(SetAutoFocusRequest { enable: true }))
                .await
                .expect_err("set autofocus must be unsupported"),
            service
                .get_auto_focus(Request::new(()))
                .await
                .expect_err("get autofocus must be unsupported"),
            service
                .set_stabilization(Request::new(SetStabilizationRequest { enable: true }))
                .await
                .expect_err("set stabilization must be unsupported"),
            service
                .get_stabilization(Request::new(()))
                .await
                .expect_err("get stabilization must be unsupported"),
        ];

        assert!(
            errors
                .iter()
                .all(|error| error.code() == Code::Unimplemented)
        );
    }
}
