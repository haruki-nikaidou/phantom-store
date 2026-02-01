use crate::entities::redis::session::SessionId;
use crate::rpc::middleware::SESSION_ID_HEADER;
use crate::services::session::{SessionService as InnerSessionService, TerminateSession};
use kanau::processor::Processor;
use phantom_shop_proto::v1::common::Empty;
use tonic::{Request, Response, Status};

pub struct SessionServiceImpl {
    pub session_service: InnerSessionService,
}

impl SessionServiceImpl {
    pub fn new(session_service: InnerSessionService) -> Self {
        Self { session_service }
    }
}

#[tonic::async_trait]
impl phantom_shop_proto::v1::auth::user::session_service_server::SessionService
    for SessionServiceImpl
{
    async fn terminate_current_session(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<Empty>, Status> {
        // Extract session ID from the authorization header
        let session_id_str = request
            .metadata()
            .get(SESSION_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;

        let session_id = SessionId::try_from_ascii_string(session_id_str)
            .map_err(|_| Status::unauthenticated("Invalid session id format"))?;

        self.session_service
            .process(TerminateSession { session_id })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }
}
