use crate::services::admin_auth::AdminAuthService as InnerService;
use phantom_shop_proto::v1::admin::{AdminLoginRequest, AdminLoginResponse};
use phantom_shop_proto::v1::common::Empty;
use tonic::{Request, Response, Status};

pub struct AdminAuthServiceImpl {
    pub inner_service: InnerService,
}

impl AdminAuthServiceImpl {
    pub fn new(inner_service: InnerService) -> Self {
        Self { inner_service }
    }
}

#[tonic::async_trait]
impl phantom_shop_proto::v1::admin::admin_auth_service_server::AdminAuthService
    for AdminAuthServiceImpl
{
    async fn admin_login(
        &self,
        request: Request<AdminLoginRequest>,
    ) -> Result<Response<AdminLoginResponse>, Status> {
        todo!()
    }

    async fn admin_logout(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        todo!()
    }
}
