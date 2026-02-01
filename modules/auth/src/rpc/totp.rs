use crate::rpc::middleware::UserId;
use crate::services::mfa::{
    CheckMfaEnabled, FinishConfiguringTotp, FinishConfiguringTotpResult, MfaService, RemoveMfa,
    StartConfiguringTotp,
};
use kanau::processor::Processor;
use phantom_shop_proto::v1::auth::common::UserTotpStatus;
use phantom_shop_proto::v1::auth::user::{
    FinishTotpSetupRequest, FinishTotpSetupResponse, FinishTotpSetupResult, StartTotpSetupRequest,
    StartTotpSetupResponse,
};
use phantom_shop_proto::v1::common::Empty;
use tonic::{Request, Response, Status};

pub struct TotpServiceImpl {
    pub mfa_service: MfaService,
}

impl TotpServiceImpl {
    pub fn new(mfa_service: MfaService) -> Self {
        Self { mfa_service }
    }
}

#[tonic::async_trait]
impl phantom_shop_proto::v1::auth::user::totp_service_server::TotpService for TotpServiceImpl {
    async fn show_totp_status(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<UserTotpStatus>, Status> {
        let user_id = UserId::read_from_request(&request)?;

        let has_totp = self
            .mfa_service
            .process(CheckMfaEnabled {
                user_id: user_id.into_inner(),
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UserTotpStatus { has_totp }))
    }

    async fn remove_totp(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let user_id = UserId::read_from_request(&request)?;

        self.mfa_service
            .process(RemoveMfa {
                user_id: user_id.into_inner(),
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn start_totp_setup(
        &self,
        request: Request<StartTotpSetupRequest>,
    ) -> Result<Response<StartTotpSetupResponse>, Status> {
        let (user_id, req) = UserId::from_request(request)?;

        let sudo_token_bytes: [u8; 16] = req
            .sudo_token
            .ok_or_else(|| Status::invalid_argument("Missing sudo token"))?
            .token
            .try_into()
            .map_err(|_| Status::invalid_argument("Invalid sudo token length"))?;

        let pending_setup = self
            .mfa_service
            .process(StartConfiguringTotp {
                user_id: user_id.into_inner(),
                sudo_token: sudo_token_bytes,
            })
            .await
            .map_err(|e| match e {
                framework::Error::PermissionsDenied => {
                    Status::permission_denied("Sudo verification failed")
                }
                _ => Status::internal(e.to_string()),
            })?;

        Ok(Response::new(StartTotpSetupResponse {
            secret: pending_setup.secret.to_vec(),
        }))
    }

    async fn finish_totp_setup(
        &self,
        request: Request<FinishTotpSetupRequest>,
    ) -> Result<Response<FinishTotpSetupResponse>, Status> {
        let (user_id, req) = UserId::from_request(request)?;

        let result = self
            .mfa_service
            .process(FinishConfiguringTotp {
                user_id: user_id.into_inner(),
                code: req.totp_code,
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_result = match result {
            FinishConfiguringTotpResult::Success => FinishTotpSetupResult::Success,
            FinishConfiguringTotpResult::InvalidCode => {
                FinishTotpSetupResult::InvalidCode
            }
            FinishConfiguringTotpResult::Duplicate => {
                FinishTotpSetupResult::Duplicate
            }
            FinishConfiguringTotpResult::Expired => FinishTotpSetupResult::Expired,
        };

        Ok(Response::new(FinishTotpSetupResponse {
            result: proto_result.into(),
        }))
    }
}
