//! Type conversions between service/entity types and proto types.

use crate::entities::db::oauth_account::OAuthAccount;
use crate::services::email_provider::{
    ChangeEmailAddressResult as ServiceChangeEmailAddressResult,
    ChangePasswordResult as ServiceChangePasswordResult, EmailLoginResult, RegisterUserResult,
    RemovePasswordResult as ServiceRemovePasswordResult, ResetPasswordResult,
    SendPasswordResetEmailResult, SendRegisterEmailResult,
};
use crate::services::mfa::{FinishConfiguringTotpResult, VerifyAndEnterSudoResult};
use crate::services::oauth_provider::{
    CreateOAuthChallengeResult, LinkOAuthAccountResult, OAuthLoginResult,
    UnlinkOAuthAccountResult as ServiceUnlinkOAuthAccountResult,
};
use crate::utils::oauth::providers::OAuthProviderName;
use phantom_shop_proto::v1::auth::common::{
    EmailSendResult, LoginResult, OAuthAccount as ProtoOAuthAccount,
    OAuthProviderName as ProtoOAuthProviderName,
};
use phantom_shop_proto::v1::auth::user as user_proto;

impl From<OAuthProviderName> for ProtoOAuthProviderName {
    fn from(name: OAuthProviderName) -> Self {
        match name {
            OAuthProviderName::Google => ProtoOAuthProviderName::OauthProviderNameGoogle,
            OAuthProviderName::Microsoft => ProtoOAuthProviderName::OauthProviderNameMicrosoft,
            OAuthProviderName::Github => ProtoOAuthProviderName::OauthProviderNameGithub,
            OAuthProviderName::Discord => ProtoOAuthProviderName::OauthProviderNameDiscord,
        }
    }
}

impl From<ProtoOAuthProviderName> for OAuthProviderName {
    fn from(name: ProtoOAuthProviderName) -> Self {
        match name {
            ProtoOAuthProviderName::OauthProviderNameGoogle => OAuthProviderName::Google,
            ProtoOAuthProviderName::OauthProviderNameMicrosoft => OAuthProviderName::Microsoft,
            ProtoOAuthProviderName::OauthProviderNameGithub => OAuthProviderName::Github,
            ProtoOAuthProviderName::OauthProviderNameDiscord => OAuthProviderName::Discord,
        }
    }
}

impl From<OAuthAccount> for ProtoOAuthAccount {
    fn from(account: OAuthAccount) -> Self {
        ProtoOAuthAccount {
            id: account.id,
            user_id: account.user_id.to_string(),
            provider_name: ProtoOAuthProviderName::from(account.provider_name).into(),
            registered_at: account.registered_at.assume_utc().unix_timestamp(),
            token_updated_at: account.token_updated_at.assume_utc().unix_timestamp(),
        }
    }
}

impl From<ServiceChangePasswordResult> for user_proto::ChangePasswordResult {
    fn from(result: ServiceChangePasswordResult) -> Self {
        match result {
            ServiceChangePasswordResult::Success => user_proto::ChangePasswordResult::Success,
            ServiceChangePasswordResult::SudoFailed => user_proto::ChangePasswordResult::SudoFailed,
            ServiceChangePasswordResult::NotFound => user_proto::ChangePasswordResult::NotFound,
        }
    }
}

impl From<ServiceRemovePasswordResult> for user_proto::RemovePasswordResult {
    fn from(result: ServiceRemovePasswordResult) -> Self {
        match result {
            ServiceRemovePasswordResult::Success => user_proto::RemovePasswordResult::Success,
            ServiceRemovePasswordResult::SudoFailed => user_proto::RemovePasswordResult::SudoFailed,
            ServiceRemovePasswordResult::AlreadyRemoved => {
                user_proto::RemovePasswordResult::AlreadyRemoved
            }
            ServiceRemovePasswordResult::NotFound => user_proto::RemovePasswordResult::NotFound,
        }
    }
}

impl From<ServiceUnlinkOAuthAccountResult> for user_proto::UnlinkOAuthAccountResult {
    fn from(result: ServiceUnlinkOAuthAccountResult) -> Self {
        match result {
            ServiceUnlinkOAuthAccountResult::Success => {
                user_proto::UnlinkOAuthAccountResult::UnlinkOauthAccountResultSuccess
            }
            ServiceUnlinkOAuthAccountResult::SudoFailed => {
                user_proto::UnlinkOAuthAccountResult::UnlinkOauthAccountResultSudoFailed
            }
            ServiceUnlinkOAuthAccountResult::NotFound => {
                user_proto::UnlinkOAuthAccountResult::UnlinkOauthAccountResultNotFound
            }
        }
    }
}

impl From<ServiceChangeEmailAddressResult> for user_proto::ChangeEmailAddressResult {
    fn from(result: ServiceChangeEmailAddressResult) -> Self {
        match result {
            ServiceChangeEmailAddressResult::Success => {
                user_proto::ChangeEmailAddressResult::Success
            }
            ServiceChangeEmailAddressResult::SudoFailed => {
                user_proto::ChangeEmailAddressResult::SudoFailed
            }
            ServiceChangeEmailAddressResult::InvalidEmail => {
                user_proto::ChangeEmailAddressResult::InvalidEmail
            }
            ServiceChangeEmailAddressResult::InvalidOtp => {
                user_proto::ChangeEmailAddressResult::InvalidOtp
            }
            ServiceChangeEmailAddressResult::EmailAddressDuplicated => {
                user_proto::ChangeEmailAddressResult::EmailAddressDuplicated
            }
            ServiceChangeEmailAddressResult::NotFound => {
                user_proto::ChangeEmailAddressResult::NotFound
            }
        }
    }
}

impl From<SendPasswordResetEmailResult> for EmailSendResult {
    fn from(result: SendPasswordResetEmailResult) -> Self {
        match result {
            SendPasswordResetEmailResult::MaybeSent => EmailSendResult::Sent,
            SendPasswordResetEmailResult::InvalidEmailAddress => EmailSendResult::NotAllowed,
            SendPasswordResetEmailResult::RateLimited => EmailSendResult::RateLimited,
        }
    }
}

impl From<SendRegisterEmailResult> for EmailSendResult {
    fn from(result: SendRegisterEmailResult) -> Self {
        match result {
            SendRegisterEmailResult::Sent => EmailSendResult::Sent,
            SendRegisterEmailResult::InvalidEmailAddress => EmailSendResult::NotAllowed,
            SendRegisterEmailResult::DuplicatedEmail => EmailSendResult::NotAllowed,
            SendRegisterEmailResult::RateLimited => EmailSendResult::RateLimited,
        }
    }
}

impl From<LinkOAuthAccountResult> for user_proto::OAuthAccountLinkingResult {
    fn from(result: LinkOAuthAccountResult) -> Self {
        match result {
            LinkOAuthAccountResult::Success => {
                user_proto::OAuthAccountLinkingResult::OauthAccountLinkingResultSuccess
            }
            LinkOAuthAccountResult::AlreadyExists => {
                user_proto::OAuthAccountLinkingResult::OauthAccountLinkingResultDuplicated
            }
            LinkOAuthAccountResult::InvalidState | LinkOAuthAccountResult::UserMismatch => {
                user_proto::OAuthAccountLinkingResult::OauthAccountLinkingResultNotFound
            }
        }
    }
}

impl From<FinishConfiguringTotpResult> for user_proto::FinishTotpSetupResult {
    fn from(result: FinishConfiguringTotpResult) -> Self {
        match result {
            FinishConfiguringTotpResult::Success => user_proto::FinishTotpSetupResult::Success,
            FinishConfiguringTotpResult::InvalidCode => {
                user_proto::FinishTotpSetupResult::InvalidCode
            }
            FinishConfiguringTotpResult::Duplicate => user_proto::FinishTotpSetupResult::Duplicate,
            FinishConfiguringTotpResult::Expired => user_proto::FinishTotpSetupResult::Expired,
        }
    }
}

impl From<CreateOAuthChallengeResult> for user_proto::CreateOAuthLinkingChallengeResponse {
    fn from(result: CreateOAuthChallengeResult) -> Self {
        match result {
            CreateOAuthChallengeResult::Redirect(url) => {
                user_proto::CreateOAuthLinkingChallengeResponse {
                    result: Some(
                        user_proto::create_o_auth_linking_challenge_response::Result::RedirectUrl(
                            url.to_string(),
                        ),
                    ),
                }
            }
            CreateOAuthChallengeResult::ProviderNotSupported => {
                user_proto::CreateOAuthLinkingChallengeResponse {
                    result: Some(
                        user_proto::create_o_auth_linking_challenge_response::Result::Error(
                            user_proto::CreateOAuthLinkingChallengeError::CreateOauthLinkingChallengeErrorProviderNotSupported.into(),
                        ),
                    ),
                }
            }
            CreateOAuthChallengeResult::SudoFailed => {
                user_proto::CreateOAuthLinkingChallengeResponse {
                    result: Some(
                        user_proto::create_o_auth_linking_challenge_response::Result::Error(
                            user_proto::CreateOAuthLinkingChallengeError::CreateOauthLinkingChallengeErrorSudoFailed.into(),
                        ),
                    ),
                }
            }
            CreateOAuthChallengeResult::UnmatchedAction => {
                user_proto::CreateOAuthLinkingChallengeResponse {
                    result: Some(
                        user_proto::create_o_auth_linking_challenge_response::Result::Error(
                            user_proto::CreateOAuthLinkingChallengeError::CreateOauthLinkingChallengeErrorSudoFailed.into(),
                        ),
                    ),
                }
            }
        }
    }
}

impl From<VerifyAndEnterSudoResult> for user_proto::EnterSudoModeResponse {
    fn from(result: VerifyAndEnterSudoResult) -> Self {
        match result {
            VerifyAndEnterSudoResult::Success(token) => user_proto::EnterSudoModeResponse {
                result: Some(user_proto::enter_sudo_mode_response::Result::SudoToken(
                    phantom_shop_proto::v1::auth::common::SudoToken {
                        token: token.to_vec(),
                    },
                )),
            },
            VerifyAndEnterSudoResult::InvalidCredential => user_proto::EnterSudoModeResponse {
                result: Some(user_proto::enter_sudo_mode_response::Result::Error(
                    user_proto::EnterSudoModeError::InvalidCredential.into(),
                )),
            },
            VerifyAndEnterSudoResult::MethodNotAllowed => user_proto::EnterSudoModeResponse {
                result: Some(user_proto::enter_sudo_mode_response::Result::Error(
                    user_proto::EnterSudoModeError::MethodNotAllowed.into(),
                )),
            },
        }
    }
}

impl From<RegisterUserResult> for user_proto::RegisterEmailAccountResponse {
    fn from(result: RegisterUserResult) -> Self {
        match result {
            RegisterUserResult::Registered => user_proto::RegisterEmailAccountResponse {
                login_result: LoginResult::Success.into(),
                user_account: None,
                session_id: None,
            },
            RegisterUserResult::RegisteredWithSession(session_id) => {
                user_proto::RegisterEmailAccountResponse {
                    login_result: LoginResult::Success.into(),
                    user_account: None,
                    session_id: Some(session_id.to_ascii_string()),
                }
            }
        }
    }
}

impl From<EmailLoginResult> for user_proto::EmailPasswordLoginResponse {
    fn from(result: EmailLoginResult) -> Self {
        match result {
            EmailLoginResult::Success(session_id) => user_proto::EmailPasswordLoginResponse {
                login_result: LoginResult::Success.into(),
                session_id: Some(session_id.to_ascii_string()),
                mfa_token: None,
            },
            EmailLoginResult::WrongCredential => user_proto::EmailPasswordLoginResponse {
                login_result: LoginResult::AccountNotFound.into(),
                session_id: None,
                mfa_token: None,
            },
            EmailLoginResult::MethodNotAvailable => user_proto::EmailPasswordLoginResponse {
                login_result: LoginResult::AccountNotFound.into(),
                session_id: None,
                mfa_token: None,
            },
            EmailLoginResult::MfaRequired(mfa_token) => user_proto::EmailPasswordLoginResponse {
                login_result: LoginResult::MfaRequired.into(),
                session_id: None,
                mfa_token: Some(mfa_token.to_vec()),
            },
        }
    }
}

impl From<ResetPasswordResult> for user_proto::ResetPasswordResponse {
    fn from(result: ResetPasswordResult) -> Self {
        match result {
            crate::services::email_provider::ResetPasswordResult::Success => {
                user_proto::ResetPasswordResponse {
                    success: true,
                    session_id: None,
                }
            }
            crate::services::email_provider::ResetPasswordResult::SuccessWithSession(
                session_id,
            ) => user_proto::ResetPasswordResponse {
                success: true,
                session_id: Some(session_id.to_ascii_string()),
            },
            crate::services::email_provider::ResetPasswordResult::InvalidOtp => {
                user_proto::ResetPasswordResponse {
                    success: false,
                    session_id: None,
                }
            }
            crate::services::email_provider::ResetPasswordResult::AccountNotFound => {
                user_proto::ResetPasswordResponse {
                    success: false,
                    session_id: None,
                }
            }
        }
    }
}

impl From<OAuthLoginResult> for user_proto::OAuthCallbackResponse {
    fn from(result: OAuthLoginResult) -> Self {
        match result {
            OAuthLoginResult::LoggedIn(session_id) => user_proto::OAuthCallbackResponse {
                result: Some(user_proto::o_auth_callback_response::Result::LoginResult(
                    user_proto::OAuthCallbackLoginBranchResult {
                        login_result: LoginResult::Success.into(),
                        session_id: Some(session_id.to_ascii_string()),
                        mfa_token: None,
                    },
                )),
            },
            OAuthLoginResult::RequiredMfa(mfa_token) => user_proto::OAuthCallbackResponse {
                result: Some(user_proto::o_auth_callback_response::Result::LoginResult(
                    user_proto::OAuthCallbackLoginBranchResult {
                        login_result: LoginResult::MfaRequired.into(),
                        session_id: None,
                        mfa_token: Some(mfa_token.to_vec()),
                    },
                )),
            },
        }
    }
}
