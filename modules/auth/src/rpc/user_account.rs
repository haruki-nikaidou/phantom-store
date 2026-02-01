use crate::entities::db::user_password::FindUserPasswordByUserId;
use crate::entities::redis::oauth_challenge::OAuthAction;
use crate::rpc::middleware::UserId;
use crate::services::email_provider::{
    ChangeEmailAddressResult as ServiceChangeEmailAddressResult, ChangePasswordResult as ServiceChangePasswordResult,
    EmailProviderService, RemovePasswordResult as ServiceRemovePasswordResult, UserChangeEmailAddress,
    UserChangePassword, UserRemovePassword,
};
use crate::services::oauth_provider::{
    CreateOAuthChallenge, CreateOAuthChallengeResult, ListOAuthAccounts, OAuthProviderService,
    UnlinkOAuthAccount as ServiceUnlinkOAuthAccount, UnlinkOAuthAccountResult as ServiceUnlinkOAuthAccountResult,
};
use crate::utils::oauth::providers::OAuthProviderName;
use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use phantom_shop_proto::v1::auth::common::{OAuthAccount as ProtoOAuthAccount, OAuthProviderName as ProtoOAuthProviderName};
use phantom_shop_proto::v1::auth::user::{
    ChangeEmailAddressRequest, ChangeEmailAddressResponse, ChangeEmailAddressResult,
    ChangePasswordRequest, ChangePasswordResponse, ChangePasswordResult,
    CreateOAuthLinkingChallengeError, CreateOAuthLinkingChallengeRequest,
    CreateOAuthLinkingChallengeResponse, RemovePasswordResponse, RemovePasswordResult,
    ShowUserAccountsInfoResponse, UnlinkOAuthAccountRequest, UnlinkOAuthAccountResponse,
    UnlinkOAuthAccountResult,
};
use phantom_shop_proto::v1::common::Empty;
use tonic::{Request, Response, Status};
use url::Url;

pub struct UserAccountServiceImpl {
    pub email_provider_service: EmailProviderService,
    pub oauth_provider_service: OAuthProviderService,
    pub db: DatabaseProcessor,
}

impl UserAccountServiceImpl {
    pub fn new(
        email_provider_service: EmailProviderService,
        oauth_provider_service: OAuthProviderService,
        db: DatabaseProcessor,
    ) -> Self {
        Self {
            email_provider_service,
            oauth_provider_service,
            db,
        }
    }
}

fn convert_provider_name(name: OAuthProviderName) -> ProtoOAuthProviderName {
    match name {
        OAuthProviderName::Google => ProtoOAuthProviderName::OauthProviderNameGoogle,
        OAuthProviderName::Microsoft => ProtoOAuthProviderName::OauthProviderNameMicrosoft,
        OAuthProviderName::Github => ProtoOAuthProviderName::OauthProviderNameGithub,
        OAuthProviderName::Discord => ProtoOAuthProviderName::OauthProviderNameDiscord,
    }
}

fn convert_proto_provider_name(name: ProtoOAuthProviderName) -> OAuthProviderName {
    match name {
        ProtoOAuthProviderName::OauthProviderNameGoogle => OAuthProviderName::Google,
        ProtoOAuthProviderName::OauthProviderNameMicrosoft => OAuthProviderName::Microsoft,
        ProtoOAuthProviderName::OauthProviderNameGithub => OAuthProviderName::Github,
        ProtoOAuthProviderName::OauthProviderNameDiscord => OAuthProviderName::Discord,
    }
}

#[tonic::async_trait]
impl phantom_shop_proto::v1::auth::user::user_account_service_server::UserAccountService
    for UserAccountServiceImpl
{
    async fn show_user_accounts_info(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<ShowUserAccountsInfoResponse>, Status> {
        let user_id = UserId::read_from_request(&request)?;

        // Check if user has password
        let has_password = self
            .db
            .process(FindUserPasswordByUserId {
                user_id: user_id.into_inner(),
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .is_some();

        // Get OAuth accounts
        let oauth_accounts = self
            .oauth_provider_service
            .process(ListOAuthAccounts {
                user_id: user_id.into_inner(),
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_oauth_accounts: Vec<ProtoOAuthAccount> = oauth_accounts
            .into_iter()
            .map(|account| ProtoOAuthAccount {
                id: account.id,
                user_id: account.user_id.to_string(),
                provider_name: convert_provider_name(account.provider_name).into(),
                registered_at: account.registered_at.assume_utc().unix_timestamp(),
                token_updated_at: account.token_updated_at.assume_utc().unix_timestamp(),
            })
            .collect();

        Ok(Response::new(ShowUserAccountsInfoResponse {
            has_password,
            oauth_accounts: proto_oauth_accounts,
        }))
    }

    async fn change_password(
        &self,
        request: Request<ChangePasswordRequest>,
    ) -> Result<Response<ChangePasswordResponse>, Status> {
        let (user_id, req) = UserId::from_request(request)?;

        let sudo_token_bytes: [u8; 16] = req
            .sudo_token
            .ok_or_else(|| Status::invalid_argument("Missing sudo token"))?
            .token
            .try_into()
            .map_err(|_| Status::invalid_argument("Invalid sudo token length"))?;

        let result = self
            .email_provider_service
            .process(UserChangePassword {
                user_id: user_id.into_inner(),
                sudo_token: sudo_token_bytes,
                new_password: req.new_password,
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_result = match result {
            ServiceChangePasswordResult::Success => ChangePasswordResult::Success,
            ServiceChangePasswordResult::SudoFailed => ChangePasswordResult::SudoFailed,
            ServiceChangePasswordResult::NotFound => ChangePasswordResult::NotFound,
        };

        Ok(Response::new(ChangePasswordResponse {
            result: proto_result.into(),
        }))
    }

    async fn remove_password(
        &self,
        request: Request<phantom_shop_proto::v1::auth::common::SudoToken>,
    ) -> Result<Response<RemovePasswordResponse>, Status> {
        let user_id = UserId::read_from_request(&request)?;
        let req = request.into_inner();

        let sudo_token_bytes: [u8; 16] = req
            .token
            .try_into()
            .map_err(|_| Status::invalid_argument("Invalid sudo token length"))?;

        let result = self
            .email_provider_service
            .process(UserRemovePassword {
                user_id: user_id.into_inner(),
                sudo_token: sudo_token_bytes,
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_result = match result {
            ServiceRemovePasswordResult::Success => RemovePasswordResult::Success,
            ServiceRemovePasswordResult::SudoFailed => RemovePasswordResult::SudoFailed,
            ServiceRemovePasswordResult::AlreadyRemoved => {
                RemovePasswordResult::AlreadyRemoved
            }
            ServiceRemovePasswordResult::NotFound => RemovePasswordResult::NotFound,
        };

        Ok(Response::new(RemovePasswordResponse {
            result: proto_result.into(),
        }))
    }

    async fn create_o_auth_linking_challenge(
        &self,
        request: Request<CreateOAuthLinkingChallengeRequest>,
    ) -> Result<Response<CreateOAuthLinkingChallengeResponse>, Status> {
        let (user_id, req) = UserId::from_request(request)?;

        let provider_name = convert_proto_provider_name(req.provider_name());

        let return_to: Url = req
            .return_to
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid return_to URL"))?;

        let sudo_token: Option<[u8; 16]> = if req.sudo_token.is_empty() {
            None
        } else {
            Some(
                req.sudo_token
                    .try_into()
                    .map_err(|_| Status::invalid_argument("Invalid sudo token length"))?,
            )
        };

        let result = self
            .oauth_provider_service
            .process(CreateOAuthChallenge {
                provider_name,
                action: OAuthAction::LinkAccount {
                    user_id: user_id.into_inner(),
                },
                return_to,
                sudo_token,
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match result {
            CreateOAuthChallengeResult::Redirect(url) => {
                Ok(Response::new(CreateOAuthLinkingChallengeResponse {
                    result: Some(
                        phantom_shop_proto::v1::auth::user::create_o_auth_linking_challenge_response::Result::RedirectUrl(
                            url.to_string(),
                        ),
                    ),
                }))
            }
            CreateOAuthChallengeResult::ProviderNotSupported => {
                Ok(Response::new(CreateOAuthLinkingChallengeResponse {
                    result: Some(
                        phantom_shop_proto::v1::auth::user::create_o_auth_linking_challenge_response::Result::Error(
                            CreateOAuthLinkingChallengeError::CreateOauthLinkingChallengeErrorProviderNotSupported.into(),
                        ),
                    ),
                }))
            }
            CreateOAuthChallengeResult::SudoFailed => {
                Ok(Response::new(CreateOAuthLinkingChallengeResponse {
                    result: Some(
                        phantom_shop_proto::v1::auth::user::create_o_auth_linking_challenge_response::Result::Error(
                            CreateOAuthLinkingChallengeError::CreateOauthLinkingChallengeErrorSudoFailed.into(),
                        ),
                    ),
                }))
            }
            CreateOAuthChallengeResult::UnmatchedAction => {
                Ok(Response::new(CreateOAuthLinkingChallengeResponse {
                    result: Some(
                        phantom_shop_proto::v1::auth::user::create_o_auth_linking_challenge_response::Result::Error(
                            CreateOAuthLinkingChallengeError::CreateOauthLinkingChallengeErrorSudoFailed.into(),
                        ),
                    ),
                }))
            }
        }
    }

    async fn unlink_o_auth_account(
        &self,
        request: Request<UnlinkOAuthAccountRequest>,
    ) -> Result<Response<UnlinkOAuthAccountResponse>, Status> {
        let (user_id, req) = UserId::from_request(request)?;

        let provider_name = convert_proto_provider_name(req.provider_name());

        let sudo_token_bytes: [u8; 16] = req
            .sudo_token
            .ok_or_else(|| Status::invalid_argument("Missing sudo token"))?
            .token
            .try_into()
            .map_err(|_| Status::invalid_argument("Invalid sudo token length"))?;

        let result = self
            .oauth_provider_service
            .process(ServiceUnlinkOAuthAccount {
                user_id: user_id.into_inner(),
                sudo_token: sudo_token_bytes,
                provider_name,
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_result = match result {
            ServiceUnlinkOAuthAccountResult::Success => {
                UnlinkOAuthAccountResult::UnlinkOauthAccountResultSuccess
            }
            ServiceUnlinkOAuthAccountResult::SudoFailed => {
                UnlinkOAuthAccountResult::UnlinkOauthAccountResultSudoFailed
            }
            ServiceUnlinkOAuthAccountResult::NotFound => {
                UnlinkOAuthAccountResult::UnlinkOauthAccountResultNotFound
            }
        };

        Ok(Response::new(UnlinkOAuthAccountResponse {
            result: proto_result.into(),
        }))
    }

    async fn change_email_address(
        &self,
        request: Request<ChangeEmailAddressRequest>,
    ) -> Result<Response<ChangeEmailAddressResponse>, Status> {
        let (user_id, req) = UserId::from_request(request)?;

        let sudo_token_bytes: [u8; 16] = req
            .sudo_token
            .ok_or_else(|| Status::invalid_argument("Missing sudo token"))?
            .token
            .try_into()
            .map_err(|_| Status::invalid_argument("Invalid sudo token length"))?;

        let result = self
            .email_provider_service
            .process(UserChangeEmailAddress {
                user_id: user_id.into_inner(),
                sudo_token: sudo_token_bytes,
                new_email: req.new_email,
                otp: req.otp,
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_result = match result {
            ServiceChangeEmailAddressResult::Success => {
                ChangeEmailAddressResult::Success
            }
            ServiceChangeEmailAddressResult::SudoFailed => {
                ChangeEmailAddressResult::SudoFailed
            }
            ServiceChangeEmailAddressResult::InvalidEmail => {
                ChangeEmailAddressResult::InvalidEmail
            }
            ServiceChangeEmailAddressResult::InvalidOtp => {
                ChangeEmailAddressResult::InvalidOtp
            }
            ServiceChangeEmailAddressResult::EmailAddressDuplicated => {
                ChangeEmailAddressResult::EmailAddressDuplicated
            }
            ServiceChangeEmailAddressResult::NotFound => {
                ChangeEmailAddressResult::NotFound
            }
        };

        Ok(Response::new(ChangeEmailAddressResponse {
            result: proto_result.into(),
        }))
    }
}
