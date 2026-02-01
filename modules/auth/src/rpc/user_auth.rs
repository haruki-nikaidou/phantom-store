use crate::entities::redis::oauth_challenge::OAuthChallengeKey;
use crate::services::email_provider::{
    EmailLoginResult, EmailProviderService, LoginUserWithPassword, RegisterPasswordless,
    RegisterUser, RegisterUserResult, RegisterWithPassword, RouteRegisterUserResult,
    SendPasswordResetEmail, SendPasswordResetEmailResult, SendRegisterEmail,
    SendRegisterEmailResult,
};
use crate::services::mfa::{MfaService, VerifyMfaLogin, VerifyMfaLoginResult};
use crate::services::oauth_provider::{
    LinkOAuthAccountCallback, LinkOAuthAccountResult, OAuthCallback, OAuthCallbackRoute,
    OAuthLogin, OAuthLoginOrRegisterRoute, OAuthLoginResult, OAuthLoginRouteResult,
    OAuthProviderService, OAuthRegister,
};
use kanau::processor::Processor;
use phantom_shop_proto::v1::auth::common::{EmailSendResult, LoginResult};
use phantom_shop_proto::v1::auth::user::{
    EmailPasswordLoginRequest, EmailPasswordLoginResponse, OAuthAccountLinkingResult,
    OAuthCallbackError, OAuthCallbackLoginBranchResult, OAuthCallbackRequest,
    OAuthCallbackResponse, RegisterEmailAccountRequest, RegisterEmailAccountResponse,
    ResetPasswordRequest, ResetPasswordResponse, SendPreAuthorizeEmailOtpRequest,
    SendPreAuthorizeEmailOtpResponse, VerifyMfaTokenRequest, VerifyMfaTokenResponse,
};
use tonic::{Request, Response, Status};
use url::Url;

pub struct UserAuthServiceImpl {
    pub email_provider_service: EmailProviderService,
    pub oauth_provider_service: OAuthProviderService,
    pub mfa_service: MfaService,
}

impl UserAuthServiceImpl {
    pub fn new(
        email_provider_service: EmailProviderService,
        oauth_provider_service: OAuthProviderService,
        mfa_service: MfaService,
    ) -> Self {
        Self {
            email_provider_service,
            oauth_provider_service,
            mfa_service,
        }
    }
}

#[tonic::async_trait]
impl phantom_shop_proto::v1::auth::user::user_auth_service_server::UserAuthService
    for UserAuthServiceImpl
{
    async fn register_email_account(
        &self,
        request: Request<RegisterEmailAccountRequest>,
    ) -> Result<Response<RegisterEmailAccountResponse>, Status> {
        let req = request.into_inner();

        // First, route the registration to determine if password or passwordless
        let route_result = self
            .email_provider_service
            .process(RegisterUser {
                email: req.email,
                otp: req.otp,
                password: req.password,
                name: req.name,
                auto_login: req.auto_login,
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match route_result {
            RouteRegisterUserResult::WithPassword(register_with_password) => {
                let result = self
                    .email_provider_service
                    .process(register_with_password)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;

                match result {
                    RegisterUserResult::Registered => {
                        Ok(Response::new(RegisterEmailAccountResponse {
                            login_result: LoginResult::Success.into(),
                            user_account: None,
                            session_id: None,
                        }))
                    }
                    RegisterUserResult::RegisteredWithSession(session_id) => {
                        Ok(Response::new(RegisterEmailAccountResponse {
                            login_result: LoginResult::Success.into(),
                            user_account: None,
                            session_id: Some(session_id.to_ascii_string()),
                        }))
                    }
                }
            }
            RouteRegisterUserResult::Passwordless(register_passwordless) => {
                let result = self
                    .email_provider_service
                    .process(register_passwordless)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;

                match result {
                    RegisterUserResult::Registered => {
                        Ok(Response::new(RegisterEmailAccountResponse {
                            login_result: LoginResult::Success.into(),
                            user_account: None,
                            session_id: None,
                        }))
                    }
                    RegisterUserResult::RegisteredWithSession(session_id) => {
                        Ok(Response::new(RegisterEmailAccountResponse {
                            login_result: LoginResult::Success.into(),
                            user_account: None,
                            session_id: Some(session_id.to_ascii_string()),
                        }))
                    }
                }
            }
            RouteRegisterUserResult::InvalidOtp => {
                Ok(Response::new(RegisterEmailAccountResponse {
                    login_result: LoginResult::AccountNotFound.into(),
                    user_account: None,
                    session_id: None,
                }))
            }
            RouteRegisterUserResult::DuplicatedEmail => {
                Ok(Response::new(RegisterEmailAccountResponse {
                    login_result: LoginResult::Blocked.into(),
                    user_account: None,
                    session_id: None,
                }))
            }
        }
    }

    async fn send_pre_authorize_email_otp(
        &self,
        request: Request<SendPreAuthorizeEmailOtpRequest>,
    ) -> Result<Response<SendPreAuthorizeEmailOtpResponse>, Status> {
        let req = request.into_inner();

        // Only handle registration and password reset OTP
        // Login OTP requires a private processor that isn't available
        let usage = req.usage();
        match usage {
            phantom_shop_proto::v1::auth::common::EmailOtpUsage::PasswordReset => {
                let result = self
                    .email_provider_service
                    .process(SendPasswordResetEmail { email: req.email })
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;

                let proto_result = match result {
                    SendPasswordResetEmailResult::MaybeSent => EmailSendResult::Sent,
                    SendPasswordResetEmailResult::InvalidEmailAddress => {
                        EmailSendResult::NotAllowed
                    }
                    SendPasswordResetEmailResult::RateLimited => EmailSendResult::RateLimited,
                };

                Ok(Response::new(SendPreAuthorizeEmailOtpResponse {
                    result: proto_result.into(),
                }))
            }
            phantom_shop_proto::v1::auth::common::EmailOtpUsage::EmailOtpRegistration => {
                let result = self
                    .email_provider_service
                    .process(SendRegisterEmail { email: req.email })
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;

                let proto_result = match result {
                    SendRegisterEmailResult::Sent => EmailSendResult::Sent,
                    SendRegisterEmailResult::InvalidEmailAddress => EmailSendResult::NotAllowed,
                    SendRegisterEmailResult::DuplicatedEmail => EmailSendResult::NotAllowed,
                    SendRegisterEmailResult::RateLimited => EmailSendResult::RateLimited,
                };

                Ok(Response::new(SendPreAuthorizeEmailOtpResponse {
                    result: proto_result.into(),
                }))
            }
            _ => {
                // Login OTP is not implemented (private processor)
                Err(Status::unimplemented("This OTP usage is not supported"))
            }
        }
    }

    async fn email_otp_login(
        &self,
        _request: Request<phantom_shop_proto::v1::auth::user::EmailOtpLoginRequest>,
    ) -> Result<Response<phantom_shop_proto::v1::auth::user::EmailOtpLoginResponse>, Status> {
        // EmailOtpLogin requires a private processor (SendEmailOtp is private)
        Err(Status::unimplemented(
            "Email OTP login is not implemented - requires public OTP login processor",
        ))
    }

    async fn email_password_login(
        &self,
        request: Request<EmailPasswordLoginRequest>,
    ) -> Result<Response<EmailPasswordLoginResponse>, Status> {
        let req = request.into_inner();

        let result = self
            .email_provider_service
            .process(LoginUserWithPassword {
                email: req.email,
                password: req.password,
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match result {
            EmailLoginResult::Success(session_id) => {
                Ok(Response::new(EmailPasswordLoginResponse {
                    login_result: LoginResult::Success.into(),
                    session_id: Some(session_id.to_ascii_string()),
                    mfa_token: None,
                }))
            }
            EmailLoginResult::WrongCredential => Ok(Response::new(EmailPasswordLoginResponse {
                login_result: LoginResult::AccountNotFound.into(),
                session_id: None,
                mfa_token: None,
            })),
            EmailLoginResult::MethodNotAvailable => Ok(Response::new(EmailPasswordLoginResponse {
                login_result: LoginResult::AccountNotFound.into(),
                session_id: None,
                mfa_token: None,
            })),
            EmailLoginResult::MfaRequired(mfa_token) => {
                Ok(Response::new(EmailPasswordLoginResponse {
                    login_result: LoginResult::MfaRequired.into(),
                    session_id: None,
                    mfa_token: Some(mfa_token.to_vec()),
                }))
            }
        }
    }

    async fn reset_password(
        &self,
        request: Request<ResetPasswordRequest>,
    ) -> Result<Response<ResetPasswordResponse>, Status> {
        let req = request.into_inner();

        let result = self
            .email_provider_service
            .process(crate::services::email_provider::ResetPassword {
                email: req.email,
                otp: req.otp,
                new_password: req.new_password,
                auto_login: req.auto_login,
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match result {
            crate::services::email_provider::ResetPasswordResult::Success => {
                Ok(Response::new(ResetPasswordResponse {
                    success: true,
                    session_id: None,
                }))
            }
            crate::services::email_provider::ResetPasswordResult::SuccessWithSession(
                session_id,
            ) => Ok(Response::new(ResetPasswordResponse {
                success: true,
                session_id: Some(session_id.to_ascii_string()),
            })),
            crate::services::email_provider::ResetPasswordResult::InvalidOtp => {
                Ok(Response::new(ResetPasswordResponse {
                    success: false,
                    session_id: None,
                }))
            }
            crate::services::email_provider::ResetPasswordResult::AccountNotFound => {
                Ok(Response::new(ResetPasswordResponse {
                    success: false,
                    session_id: None,
                }))
            }
        }
    }

    async fn verify_mfa_token(
        &self,
        request: Request<VerifyMfaTokenRequest>,
    ) -> Result<Response<VerifyMfaTokenResponse>, Status> {
        let req = request.into_inner();

        let mfa_token: [u8; 32] = req
            .mfa_token
            .try_into()
            .map_err(|_| Status::invalid_argument("Invalid MFA token length"))?;

        let code: u32 = req
            .mfa_code
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid MFA code format"))?;

        let result = self
            .mfa_service
            .process(VerifyMfaLogin { mfa_token, code })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match result {
            VerifyMfaLoginResult::Success(session_id) => {
                Ok(Response::new(VerifyMfaTokenResponse {
                    success: true,
                    session_id: Some(session_id.to_ascii_string()),
                }))
            }
            VerifyMfaLoginResult::InvalidToken
            | VerifyMfaLoginResult::InvalidCode
            | VerifyMfaLoginResult::NoNeed => Ok(Response::new(VerifyMfaTokenResponse {
                success: false,
                session_id: None,
            })),
        }
    }

    async fn o_auth_callback(
        &self,
        request: Request<OAuthCallbackRequest>,
    ) -> Result<Response<OAuthCallbackResponse>, Status> {
        let req = request.into_inner();

        let state: OAuthChallengeKey = req
            .state
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid OAuth state"))?;

        let redirect_uri: Url = req
            .redirect_uri
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid redirect URI"))?;

        // Route the OAuth callback
        let route_result = self
            .oauth_provider_service
            .process(OAuthCallback {
                code: req.code.clone(),
                state,
                redirect_uri: redirect_uri.clone(),
                user_id: None, // No authenticated user for login flow
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match route_result {
            OAuthCallbackRoute::LoginOrRegister(login_or_register) => {
                // Determine if login or register
                let login_route_result = self
                    .oauth_provider_service
                    .process(login_or_register)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;

                match login_route_result {
                    OAuthLoginRouteResult::Login(oauth_login) => {
                        let login_result = self
                            .oauth_provider_service
                            .process(oauth_login)
                            .await
                            .map_err(|e| Status::internal(e.to_string()))?;

                        match login_result {
                            OAuthLoginResult::LoggedIn(session_id) => {
                                Ok(Response::new(OAuthCallbackResponse {
                                    result: Some(
                                        phantom_shop_proto::v1::auth::user::o_auth_callback_response::Result::LoginResult(
                                            OAuthCallbackLoginBranchResult {
                                                login_result: LoginResult::Success.into(),
                                                session_id: Some(session_id.to_ascii_string()),
                                                mfa_token: None,
                                            },
                                        ),
                                    ),
                                }))
                            }
                            OAuthLoginResult::RequiredMfa(mfa_token) => {
                                Ok(Response::new(OAuthCallbackResponse {
                                    result: Some(
                                        phantom_shop_proto::v1::auth::user::o_auth_callback_response::Result::LoginResult(
                                            OAuthCallbackLoginBranchResult {
                                                login_result: LoginResult::MfaRequired.into(),
                                                session_id: None,
                                                mfa_token: Some(mfa_token.to_vec()),
                                            },
                                        ),
                                    ),
                                }))
                            }
                        }
                    }
                    OAuthLoginRouteResult::Register(oauth_register) => {
                        let register_result = self
                            .oauth_provider_service
                            .process(oauth_register)
                            .await
                            .map_err(|e| Status::internal(e.to_string()))?;

                        Ok(Response::new(OAuthCallbackResponse {
                            result: Some(
                                phantom_shop_proto::v1::auth::user::o_auth_callback_response::Result::LoginResult(
                                    OAuthCallbackLoginBranchResult {
                                        login_result: LoginResult::Success.into(),
                                        session_id: Some(register_result.session_id.to_ascii_string()),
                                        mfa_token: None,
                                    },
                                ),
                            ),
                        }))
                    }
                }
            }
            OAuthCallbackRoute::LinkAccount(link_account) => {
                let result = self
                    .oauth_provider_service
                    .process(link_account)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;

                let proto_result = match result {
                    LinkOAuthAccountResult::Success => {
                        OAuthAccountLinkingResult::OauthAccountLinkingResultSuccess
                    }
                    LinkOAuthAccountResult::AlreadyExists => {
                        OAuthAccountLinkingResult::OauthAccountLinkingResultDuplicated
                    }
                    LinkOAuthAccountResult::InvalidState | LinkOAuthAccountResult::UserMismatch => {
                        OAuthAccountLinkingResult::OauthAccountLinkingResultNotFound
                    }
                };

                Ok(Response::new(OAuthCallbackResponse {
                    result: Some(
                        phantom_shop_proto::v1::auth::user::o_auth_callback_response::Result::AccountLinkingResult(
                            proto_result.into(),
                        ),
                    ),
                }))
            }
            OAuthCallbackRoute::Unmatched => Ok(Response::new(OAuthCallbackResponse {
                result: Some(
                    phantom_shop_proto::v1::auth::user::o_auth_callback_response::Result::Error(
                        OAuthCallbackError::OauthCallbackErrorActionMismatch.into(),
                    ),
                ),
            })),
            OAuthCallbackRoute::InvalidState => Ok(Response::new(OAuthCallbackResponse {
                result: Some(
                    phantom_shop_proto::v1::auth::user::o_auth_callback_response::Result::Error(
                        OAuthCallbackError::OauthCallbackErrorInvalidState.into(),
                    ),
                ),
            })),
        }
    }
}
