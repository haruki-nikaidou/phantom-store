use crate::config::AuthConfig;
use crate::entities::db::oauth_account::{
    AppendOAuthAccount, DeleteOAuthAccountById, FindOAuthAccountByProviderUserId,
    FindOAuthAccountsByUserId, OAuthAccount, RegisterOAuthAccount,
};
use crate::entities::redis::oauth_challenge::{OAuthAction, OAuthChallenge, OAuthChallengeKey};
use crate::entities::redis::session::SessionId;
use crate::services::mfa::{CheckMfaEnabled, CreateLoginMfaSession, MfaService, VerifySudoToken};
use crate::services::session::{CreateSession, SessionService};
use crate::utils::oauth::OAuthUserInfo;
use crate::utils::oauth::providers::{OAuthDataFetch, OAuthProviderName};
use admin::utils::config_provider::find_config_from_redis;
use framework::rabbitmq::AmqpPool;
use framework::redis::{KeyValueWrite, RedisConnection};
use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use oauth2::TokenResponse;
use url::Url;
use uuid::Uuid;

#[derive(Clone)]
pub struct OAuthProviderService {
    pub db: DatabaseProcessor,
    pub config_store: RedisConnection,
    pub redis: RedisConnection,
    pub mq: AmqpPool,
    pub session_service: SessionService,
    pub mfa_service: MfaService,
}

#[derive(Debug, Clone)]
pub struct CreateOAuthChallenge {
    pub provider_name: OAuthProviderName,
    pub action: OAuthAction,
    pub return_to: Url,
    pub sudo_token: Option<[u8; 16]>,
}

#[derive(Debug, Clone)]
pub enum CreateOAuthChallengeResult {
    Redirect(Url),
    ProviderNotSupported,
    SudoFailed,
}

impl Processor<CreateOAuthChallenge> for OAuthProviderService {
    type Output = CreateOAuthChallengeResult;
    type Error = framework::Error;
    async fn process(
        &self,
        input: CreateOAuthChallenge,
    ) -> Result<CreateOAuthChallengeResult, framework::Error> {
        let mut redis = self.config_store.clone();
        let config = find_config_from_redis::<AuthConfig>(&mut redis).await?;

        // Find the provider config
        let Some(provider_config) = config
            .oauth_providers
            .providers
            .iter()
            .find(|p| p.name == input.provider_name)
        else {
            return Ok(CreateOAuthChallengeResult::ProviderNotSupported);
        };

        // If action is Bind, verify sudo token
        if let OAuthAction::Bind { user_id } = &input.action {
            if let Some(sudo_token) = input.sudo_token {
                let verified = self
                    .mfa_service
                    .process(VerifySudoToken {
                        user_id: *user_id,
                        token: sudo_token,
                    })
                    .await?;
                if !verified {
                    return Ok(CreateOAuthChallengeResult::SudoFailed);
                }
            } else {
                return Ok(CreateOAuthChallengeResult::SudoFailed);
            }
        }

        // Build OAuth client
        let constants = input.provider_name.oauth_constants();
        let redirect_uri = input.return_to.as_str();
        let client = constants
            .build_oauth_client(&provider_config.config, redirect_uri)
            .map_err(|e| framework::Error::BusinessPanic(anyhow::anyhow!("Invalid URL: {e}")))?;

        // Generate authorization URL
        let auth_essentials = constants.full_authorize_url(&client);

        // Create and store OAuth challenge
        let challenge = OAuthChallenge {
            state: auth_essentials.state,
            provider_name: input.provider_name,
            action: input.action,
            redirect_uri: redirect_uri.to_string(),
            pkce_verifier: auth_essentials.pkce_verifier,
        };

        let challenge_ttl: std::time::Duration =
            config.oauth_providers.challenge_expiration.try_into().map_err(|e| {
                framework::Error::BusinessPanic(anyhow::anyhow!(
                    "Invalid challenge expiration: {e}"
                ))
            })?;

        let mut redis = self.redis.clone();
        challenge.write_with_ttl(&mut redis, challenge_ttl).await?;

        Ok(CreateOAuthChallengeResult::Redirect(
            auth_essentials.authorization_url,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct OAuthCallback {
    pub code: String,
    pub state: OAuthChallengeKey,
    pub redirect_uri: Url,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub enum OAuthCallbackRoute {
    LoginOrRegister(OAuthLoginOrRegisterRoute),
    LinkAccount(LinkOAuthAccountCallback),
    Unmatched,
    InvalidState,
}

impl Processor<OAuthCallback> for OAuthProviderService {
    type Output = OAuthCallbackRoute;
    type Error = framework::Error;
    async fn process(&self, input: OAuthCallback) -> Result<OAuthCallbackRoute, framework::Error> {
        let mut redis = self.redis.clone();

        // Atomically read and delete the challenge from Redis
        let Some(challenge) = OAuthChallenge::read_and_delete(&mut redis, input.state).await?
        else {
            return Ok(OAuthCallbackRoute::InvalidState);
        };

        match challenge.action {
            OAuthAction::Login => {
                // For login/register flow, pass the full challenge data
                Ok(OAuthCallbackRoute::LoginOrRegister(
                    OAuthLoginOrRegisterRoute {
                        code: input.code,
                        challenge,
                    },
                ))
            }
            OAuthAction::Bind { user_id } => {
                // Validate that the user_id matches if provided
                if let Some(input_user_id) = input.user_id {
                    if input_user_id != user_id {
                        return Ok(OAuthCallbackRoute::Unmatched);
                    }
                }
                Ok(OAuthCallbackRoute::LinkAccount(LinkOAuthAccountCallback {
                    user_id,
                    challenge,
                    code: input.code,
                    return_to: input.redirect_uri.to_string(),
                }))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct OAuthLoginOrRegisterRoute {
    pub code: String,
    pub challenge: OAuthChallenge,
}

#[derive(Debug, Clone)]
pub enum OAuthLoginRouteResult {
    Login(OAuthLogin),
    Register(OAuthRegister),
}

impl Processor<OAuthLoginOrRegisterRoute> for OAuthProviderService {
    type Output = OAuthLoginRouteResult;
    type Error = framework::Error;
    async fn process(
        &self,
        input: OAuthLoginOrRegisterRoute,
    ) -> Result<OAuthLoginRouteResult, framework::Error> {
        let mut redis = self.config_store.clone();
        let config = find_config_from_redis::<AuthConfig>(&mut redis).await?;

        // Find the provider config
        let provider_config = config
            .oauth_providers
            .providers
            .iter()
            .find(|p| p.name == input.challenge.provider_name)
            .ok_or_else(|| {
                framework::Error::BusinessPanic(anyhow::anyhow!(
                    "Provider not found: {:?}",
                    input.challenge.provider_name
                ))
            })?;

        // Build OAuth client
        let constants = input.challenge.provider_name.oauth_constants();
        let client = constants
            .build_oauth_client(&provider_config.config, &input.challenge.redirect_uri)
            .map_err(|e| framework::Error::BusinessPanic(anyhow::anyhow!("Invalid URL: {e}")))?;

        // Exchange the authorization code for tokens
        let token_response = constants
            .exchange_code(
                &client,
                &input.code,
                input.challenge.pkce_verifier.as_deref(),
            )
            .await
            .map_err(|e| {
                framework::Error::BusinessPanic(anyhow::anyhow!("Token exchange failed: {e}"))
            })?;

        let access_token = token_response.access_token().secret().to_string();
        let refresh_token = token_response
            .refresh_token()
            .map(|t| t.secret().to_string());

        // Fetch user info from the OAuth provider
        let user_info = input
            .challenge
            .provider_name
            .fetch_user_info(&access_token)
            .await
            .map_err(|e| {
                framework::Error::BusinessPanic(anyhow::anyhow!("Failed to fetch user info: {e}"))
            })?;

        // Check if OAuth account already exists
        let existing_account = self
            .db
            .process(FindOAuthAccountByProviderUserId {
                provider_name: input.challenge.provider_name,
                provider_user_id: user_info.id.to_string(),
            })
            .await?;

        if let Some(account) = existing_account {
            // Existing user - login
            Ok(OAuthLoginRouteResult::Login(OAuthLogin {
                user_id: account.user_id,
                provider_name: input.challenge.provider_name,
            }))
        } else {
            // New user - register
            Ok(OAuthLoginRouteResult::Register(OAuthRegister {
                provider_name: input.challenge.provider_name,
                oauth_access_token: access_token,
                oauth_refresh_token: refresh_token,
                user_info,
            }))
        }
    }
}

#[derive(Debug, Clone)]
pub struct OAuthLogin {
    pub user_id: Uuid,
    pub provider_name: OAuthProviderName,
}

#[derive(Debug, Clone)]
pub enum OAuthLoginResult {
    LoggedIn(SessionId),
    RequiredMfa([u8; 32]),
}

impl Processor<OAuthLogin> for OAuthProviderService {
    type Output = OAuthLoginResult;
    type Error = framework::Error;
    async fn process(&self, input: OAuthLogin) -> Result<OAuthLoginResult, framework::Error> {
        // Check if MFA is enabled for this user
        let mfa_enabled = self
            .mfa_service
            .process(CheckMfaEnabled {
                user_id: input.user_id,
            })
            .await?;

        if mfa_enabled {
            // MFA is required - create MFA login session token
            let mfa_token = self
                .mfa_service
                .process(CreateLoginMfaSession {
                    user_id: input.user_id,
                })
                .await?;
            Ok(OAuthLoginResult::RequiredMfa(mfa_token.token))
        } else {
            // No MFA - create session directly
            let session_id = self
                .session_service
                .process(CreateSession {
                    user_id: input.user_id,
                })
                .await?;
            Ok(OAuthLoginResult::LoggedIn(session_id))
        }
    }
}

#[derive(Debug, Clone)]
pub struct OAuthRegister {
    pub provider_name: OAuthProviderName,
    pub oauth_access_token: String,
    pub oauth_refresh_token: Option<String>,
    pub user_info: OAuthUserInfo,
}

#[derive(Debug, Clone)]
pub struct OAuthRegisterResult {
    pub user_id: Uuid,
    pub session_id: SessionId,
}

impl Processor<OAuthRegister> for OAuthProviderService {
    type Output = OAuthRegisterResult;
    type Error = framework::Error;
    async fn process(&self, input: OAuthRegister) -> Result<OAuthRegisterResult, framework::Error> {
        // Extract email (required for registration)
        let email = input
            .user_info
            .email
            .as_ref()
            .ok_or_else(|| {
                framework::Error::InvalidInput
            })?
            .to_string();

        // Register the OAuth account (creates user account in transaction)
        let oauth_account = self
            .db
            .process(RegisterOAuthAccount {
                provider_name: input.provider_name,
                provider_user_id: input.user_info.id.to_string(),
                email,
                name: input.user_info.name.as_ref().map(|n| n.to_string()),
            })
            .await?;

        // Create session for the new user
        let session_id = self
            .session_service
            .process(CreateSession {
                user_id: oauth_account.user_id,
            })
            .await?;

        Ok(OAuthRegisterResult {
            user_id: oauth_account.user_id,
            session_id,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LinkOAuthAccountCallback {
    pub user_id: Uuid,
    pub challenge: OAuthChallenge,
    pub code: String,
    pub return_to: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkOAuthAccountResult {
    Success,
    InvalidState,
    UserMismatch,
    AlreadyExists,
}

impl Processor<LinkOAuthAccountCallback> for OAuthProviderService {
    type Output = LinkOAuthAccountResult;
    type Error = framework::Error;
    async fn process(
        &self,
        input: LinkOAuthAccountCallback,
    ) -> Result<LinkOAuthAccountResult, framework::Error> {
        // Verify challenge user matches input user
        match &input.challenge.action {
            OAuthAction::Bind { user_id } if *user_id != input.user_id => {
                return Ok(LinkOAuthAccountResult::UserMismatch);
            }
            OAuthAction::Login => {
                return Ok(LinkOAuthAccountResult::InvalidState);
            }
            _ => {}
        }

        let mut redis = self.config_store.clone();
        let config = find_config_from_redis::<AuthConfig>(&mut redis).await?;

        // Find the provider config
        let provider_config = config
            .oauth_providers
            .providers
            .iter()
            .find(|p| p.name == input.challenge.provider_name)
            .ok_or_else(|| {
                framework::Error::BusinessPanic(anyhow::anyhow!(
                    "Provider not found: {:?}",
                    input.challenge.provider_name
                ))
            })?;

        // Build OAuth client
        let constants = input.challenge.provider_name.oauth_constants();
        let client = constants
            .build_oauth_client(&provider_config.config, &input.challenge.redirect_uri)
            .map_err(|e| framework::Error::BusinessPanic(anyhow::anyhow!("Invalid URL: {e}")))?;

        // Exchange the authorization code for tokens
        let token_response = constants
            .exchange_code(
                &client,
                &input.code,
                input.challenge.pkce_verifier.as_deref(),
            )
            .await
            .map_err(|e| {
                framework::Error::BusinessPanic(anyhow::anyhow!("Token exchange failed: {e}"))
            })?;

        let access_token = token_response.access_token().secret().to_string();

        // Fetch user info from the OAuth provider
        let user_info = input
            .challenge
            .provider_name
            .fetch_user_info(&access_token)
            .await
            .map_err(|e| {
                framework::Error::BusinessPanic(anyhow::anyhow!("Failed to fetch user info: {e}"))
            })?;

        // Check if this OAuth account is already linked to any user
        let existing_account = self
            .db
            .process(FindOAuthAccountByProviderUserId {
                provider_name: input.challenge.provider_name,
                provider_user_id: user_info.id.to_string(),
            })
            .await?;

        if existing_account.is_some() {
            return Ok(LinkOAuthAccountResult::AlreadyExists);
        }

        // Append the OAuth account to the user
        self.db
            .process(AppendOAuthAccount {
                user_id: input.user_id,
                provider_name: input.challenge.provider_name,
                provider_user_id: user_info.id.to_string(),
            })
            .await?;

        Ok(LinkOAuthAccountResult::Success)
    }
}

#[derive(Debug, Clone)]
pub struct UnlinkOAuthAccount {
    pub user_id: Uuid,
    pub sudo_token: [u8; 16],
    pub provider_name: OAuthProviderName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnlinkOAuthAccountResult {
    Success,
    SudoFailed,
    NotFound,
}

impl Processor<UnlinkOAuthAccount> for OAuthProviderService {
    type Output = UnlinkOAuthAccountResult;
    type Error = framework::Error;
    async fn process(
        &self,
        input: UnlinkOAuthAccount,
    ) -> Result<UnlinkOAuthAccountResult, framework::Error> {
        // Verify sudo token
        let sudo_verified = self
            .mfa_service
            .process(VerifySudoToken {
                user_id: input.user_id,
                token: input.sudo_token,
            })
            .await?;

        if !sudo_verified {
            return Ok(UnlinkOAuthAccountResult::SudoFailed);
        }

        // Find OAuth accounts for this user
        let accounts = self
            .db
            .process(FindOAuthAccountsByUserId {
                user_id: input.user_id,
            })
            .await?;

        // Find the account with the matching provider
        let account = accounts
            .iter()
            .find(|a| a.provider_name == input.provider_name);

        let Some(account) = account else {
            return Ok(UnlinkOAuthAccountResult::NotFound);
        };

        // Delete the OAuth account
        self.db
            .process(DeleteOAuthAccountById { id: account.id })
            .await?;

        Ok(UnlinkOAuthAccountResult::Success)
    }
}

#[derive(Debug, Clone)]
pub struct ListOAuthAccounts {
    pub user_id: Uuid,
}

impl Processor<ListOAuthAccounts> for OAuthProviderService {
    type Output = Vec<OAuthAccount>;
    type Error = framework::Error;
    async fn process(
        &self,
        input: ListOAuthAccounts,
    ) -> Result<Vec<OAuthAccount>, framework::Error> {
        let accounts = self
            .db
            .process(FindOAuthAccountsByUserId {
                user_id: input.user_id,
            })
            .await?;

        Ok(accounts)
    }
}
