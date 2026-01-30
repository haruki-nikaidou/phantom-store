use crate::entities::db::oauth_account::OAuthAccount;
use crate::entities::redis::oauth_challenge::{OAuthAction, OAuthChallenge, OAuthChallengeKey};
use crate::entities::redis::session::SessionId;
use crate::services::mfa::MfaService;
use crate::services::session::SessionService;
use crate::utils::oauth::OAuthUserInfo;
use crate::utils::oauth::providers::OAuthProviderName;
use framework::rabbitmq::AmqpPool;
use framework::redis::RedisConnection;
use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
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
        todo!()
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
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct OAuthLoginOrRegisterRoute {
    pub code: String,
    pub challenge: OAuthChallengeKey,
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
        todo!()
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
        todo!()
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
        todo!()
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
        todo!()
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
        todo!()
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
        todo!()
    }
}
