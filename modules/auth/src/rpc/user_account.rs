use crate::entities::db::user_password::FindUserPasswordByUserId;
use crate::entities::redis::oauth_challenge::OAuthAction;
use crate::rpc::middleware::UserId;
use crate::services::email_provider::{
    EmailProviderService, UserChangeEmailAddress, UserChangePassword, UserRemovePassword,
};
use crate::services::oauth_provider::{
    CreateOAuthChallenge, ListOAuthAccounts, OAuthProviderService,
    UnlinkOAuthAccount as ServiceUnlinkOAuthAccount,
};
use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use phantom_shop_proto::v1::auth::common::OAuthAccount as ProtoOAuthAccount;
use phantom_shop_proto::v1::auth::user as user_proto;
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

#[tonic::async_trait]
impl phantom_shop_proto::v1::auth::user::user_account_service_server::UserAccountService
    for UserAccountServiceImpl
{
    async fn show_user_accounts_info(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<user_proto::ShowUserAccountsInfoResponse>, Status> {
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

        let proto_oauth_accounts: Vec<ProtoOAuthAccount> =
            oauth_accounts.into_iter().map(Into::into).collect();

        Ok(Response::new(user_proto::ShowUserAccountsInfoResponse {
            has_password,
            oauth_accounts: proto_oauth_accounts,
        }))
    }

    async fn change_password(
        &self,
        request: Request<user_proto::ChangePasswordRequest>,
    ) -> Result<Response<user_proto::ChangePasswordResponse>, Status> {
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

        let proto_result: user_proto::ChangePasswordResult = result.into();

        Ok(Response::new(user_proto::ChangePasswordResponse {
            result: proto_result.into(),
        }))
    }

    async fn remove_password(
        &self,
        request: Request<phantom_shop_proto::v1::auth::common::SudoToken>,
    ) -> Result<Response<user_proto::RemovePasswordResponse>, Status> {
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

        let proto_result: user_proto::RemovePasswordResult = result.into();

        Ok(Response::new(user_proto::RemovePasswordResponse {
            result: proto_result.into(),
        }))
    }

    async fn create_o_auth_linking_challenge(
        &self,
        request: Request<user_proto::CreateOAuthLinkingChallengeRequest>,
    ) -> Result<Response<user_proto::CreateOAuthLinkingChallengeResponse>, Status> {
        let (user_id, req) = UserId::from_request(request)?;

        let provider_name = req.provider_name().into();

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

        let proto_result: user_proto::CreateOAuthLinkingChallengeResponse = result.into();

        Ok(Response::new(proto_result))
    }

    async fn unlink_o_auth_account(
        &self,
        request: Request<user_proto::UnlinkOAuthAccountRequest>,
    ) -> Result<Response<user_proto::UnlinkOAuthAccountResponse>, Status> {
        let (user_id, req) = UserId::from_request(request)?;

        let provider_name = req.provider_name().into();

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

        let proto_result: user_proto::UnlinkOAuthAccountResult = result.into();

        Ok(Response::new(user_proto::UnlinkOAuthAccountResponse {
            result: proto_result.into(),
        }))
    }

    async fn change_email_address(
        &self,
        request: Request<user_proto::ChangeEmailAddressRequest>,
    ) -> Result<Response<user_proto::ChangeEmailAddressResponse>, Status> {
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

        let proto_result: user_proto::ChangeEmailAddressResult = result.into();

        Ok(Response::new(user_proto::ChangeEmailAddressResponse {
            result: proto_result.into(),
        }))
    }
}
