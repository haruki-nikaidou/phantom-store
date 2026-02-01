use crate::rpc::middleware::UserId;
use crate::services::mfa::{
    ListSudoMethods as ServiceListSudoMethods, MfaService, SudoMethod, SudoVerificationMethod,
    VerifyAndEnterSudo,
};
use kanau::processor::Processor;
use phantom_shop_proto::v1::auth::user as user_proto;
use phantom_shop_proto::v1::common::Empty;
use tonic::{Request, Response, Status};

pub struct SudoServiceImpl {
    pub mfa_service: MfaService,
}

impl SudoServiceImpl {
    pub fn new(mfa_service: MfaService) -> Self {
        Self { mfa_service }
    }
}

#[tonic::async_trait]
impl phantom_shop_proto::v1::auth::user::sudo_service_server::SudoService for SudoServiceImpl {
    async fn send_email_otp(
        &self,
        _request: Request<user_proto::SendEmailOtpRequest>,
    ) -> Result<Response<user_proto::SendEmailOtpResponse>, Status> {
        // SendEmailOtp requires a private processor (SendEmailOtp is private in EmailProviderService)
        Err(Status::unimplemented(
            "Send email OTP is not implemented - requires public email OTP processor",
        ))
    }

    async fn list_sudo_methods(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<user_proto::ListSudoMethodsResponse>, Status> {
        let user_id = UserId::read_from_request(&request)?;

        let methods = self
            .mfa_service
            .process(ServiceListSudoMethods {
                user_id: user_id.into_inner(),
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let has_totp = methods.contains(&SudoMethod::Totp);
        let has_email_otp = methods.contains(&SudoMethod::Email);

        Ok(Response::new(user_proto::ListSudoMethodsResponse {
            has_totp,
            has_email_otp,
        }))
    }

    async fn enter_sudo_mode(
        &self,
        request: Request<user_proto::EnterSudoModeRequest>,
    ) -> Result<Response<user_proto::EnterSudoModeResponse>, Status> {
        let (user_id, req) = UserId::from_request(request)?;

        let method = match req.method {
            Some(
                phantom_shop_proto::v1::auth::user::enter_sudo_mode_request::Method::TotpCode(code),
            ) => SudoVerificationMethod::Totp(code),
            Some(
                phantom_shop_proto::v1::auth::user::enter_sudo_mode_request::Method::EmailOtp(otp),
            ) => SudoVerificationMethod::EmailOtp(otp),
            None => {
                return Err(Status::invalid_argument(
                    "Must provide either TOTP code or email OTP",
                ));
            }
        };

        let result = self
            .mfa_service
            .process(VerifyAndEnterSudo {
                user_id: user_id.into_inner(),
                method,
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(result.into()))
    }
}
