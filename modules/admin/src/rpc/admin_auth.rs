use crate::entities::admin_session::AdminSessionId;
use crate::rpc::middleware::ADMIN_AUTHORIZATION_HEADER;
use crate::services::admin_auth::{
    AdminAuthService as InnerService, AdminLogin, AdminLoginResult, AdminLogout,
};
use kanau::processor::Processor;
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
        let req = request.into_inner();

        let result = self
            .inner_service
            .process(AdminLogin {
                email: req.email,
                password: req.password,
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match result {
            AdminLoginResult::Success(session_id) => Ok(Response::new(AdminLoginResponse {
                session_id: session_id.to_ascii_string(),
            })),
            AdminLoginResult::WrongCredential => {
                Err(Status::unauthenticated("Invalid email or password"))
            }
        }
    }

    async fn admin_logout(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        // Extract session ID from the authorization header
        let session_id_str = request
            .metadata()
            .get(ADMIN_AUTHORIZATION_HEADER)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;

        let session_id = AdminSessionId::try_from_ascii_string(session_id_str)
            .map_err(|_| Status::unauthenticated("Invalid session id format"))?;

        self.inner_service
            .process(AdminLogout { session_id })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }
}
