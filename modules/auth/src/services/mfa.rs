use crate::config::AuthConfig;
use crate::entities::db::email_otp::{EmailOtpUsage, FindValidEmailOtp};
use crate::entities::db::totp::{CreateTotp, FindTotpByUserId, RemoveTotpByUserId};
use crate::entities::db::user_account::FindUserAccountById;
use crate::entities::redis::mfa_token::{MfaLoginToken, MfaLoginTokenKey};
use crate::entities::redis::session::SessionId;
use crate::entities::redis::sudo_token::{SudoToken, SudoTokenKey};
use crate::entities::redis::totp_setup::{PendingTotpSetup, PendingTotpSetupKey};
use crate::services::session::{CreateSession, SessionService};
use admin::utils::config_provider::find_config_from_redis;
use framework::rabbitmq::AmqpPool;
use framework::redis::{KeyValueRead, KeyValueWrite, RedisConnection};
use framework::sqlx::DatabaseProcessor;
use kanau::message::MessageDe;
use kanau::processor::Processor;
use redis::AsyncCommands;
use tracing::instrument;
use uuid::Uuid;

#[derive(Clone)]
pub struct MfaService {
    pub db: DatabaseProcessor,
    pub config_store: RedisConnection,
    pub redis: RedisConnection,
    pub mq: AmqpPool,
    pub session_service: SessionService,
}

#[derive(Debug, Clone)]
pub struct CheckMfaEnabled {
    pub user_id: Uuid,
}

impl Processor<CheckMfaEnabled> for MfaService {
    type Output = bool;
    type Error = framework::Error;
    async fn process(&self, input: CheckMfaEnabled) -> Result<bool, framework::Error> {
        let totp = self
            .db
            .process(FindTotpByUserId {
                user_id: input.user_id,
            })
            .await?;
        Ok(totp.is_some())
    }
}

#[derive(Debug, Clone)]
pub struct VerifyMfa {
    pub user_id: Uuid,
    pub code: u32,
}

impl Processor<VerifyMfa> for MfaService {
    type Output = bool;
    type Error = framework::Error;
    async fn process(&self, input: VerifyMfa) -> Result<bool, framework::Error> {
        let Some(totp) = self
            .db
            .process(FindTotpByUserId {
                user_id: input.user_id,
            })
            .await?
        else {
            return Ok(false);
        };
        Ok(verify_totp(totp.secret, input.code))
    }
}

#[derive(Debug, Clone)]
pub struct RemoveMfa {
    pub user_id: Uuid,
}

impl Processor<RemoveMfa> for MfaService {
    type Output = ();
    type Error = framework::Error;
    async fn process(&self, input: RemoveMfa) -> Result<(), framework::Error> {
        self.db
            .process(RemoveTotpByUserId {
                user_id: input.user_id,
            })
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StartConfiguringTotp {
    pub user_id: Uuid,
}

impl Processor<StartConfiguringTotp> for MfaService {
    type Output = PendingTotpSetup;
    type Error = framework::Error;
    async fn process(
        &self,
        input: StartConfiguringTotp,
    ) -> Result<PendingTotpSetup, framework::Error> {
        const RPC6238_TOTP_KEY_LENGTH: usize = 20;
        let secret: [u8; RPC6238_TOTP_KEY_LENGTH] = rand::random();
        let pending = PendingTotpSetup {
            user_id: PendingTotpSetupKey(input.user_id),
            secret: secret.into(),
        };
        let mut redis = self.redis.clone();
        let config = find_config_from_redis::<AuthConfig>(&mut redis).await?.mfa;
        let setup_ttl: std::time::Duration = config.setup_code_ttl.try_into().map_err(|e| {
            framework::Error::BusinessPanic(anyhow::anyhow!("Invalid mfa token ttl: {e}"))
        })?;
        pending.write_with_ttl(&mut redis, setup_ttl).await?;
        Ok(pending)
    }
}

#[derive(Debug, Clone)]
pub struct FinishConfiguringTotp {
    pub user_id: Uuid,
    pub code: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum FinishConfiguringTotpResult {
    Success,
    InvalidCode,
    Duplicate,
    Expired,
}

impl Processor<FinishConfiguringTotp> for MfaService {
    type Output = FinishConfiguringTotpResult;
    type Error = framework::Error;
    async fn process(
        &self,
        input: FinishConfiguringTotp,
    ) -> Result<FinishConfiguringTotpResult, framework::Error> {
        let mut redis = self.redis.clone();
        let Some(pending) =
            PendingTotpSetup::read(&mut redis, PendingTotpSetupKey(input.user_id)).await?
        else {
            return Ok(FinishConfiguringTotpResult::Expired);
        };
        let has_totp = self
            .db
            .process(FindTotpByUserId {
                user_id: input.user_id,
            })
            .await?
            .is_some();
        if has_totp {
            return Ok(FinishConfiguringTotpResult::Duplicate);
        }
        if !verify_totp(pending.secret.to_vec(), input.code) {
            return Ok(FinishConfiguringTotpResult::InvalidCode);
        }
        self.db
            .process(CreateTotp {
                user_id: input.user_id,
                secret: pending.secret.to_vec(),
            })
            .await?;
        Ok(FinishConfiguringTotpResult::Success)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SudoMethod {
    Totp,
    Email,
}

#[derive(Debug, Clone)]
pub struct ListSudoMethods {
    pub user_id: Uuid,
}

impl Processor<ListSudoMethods> for MfaService {
    type Output = Vec<SudoMethod>;
    type Error = framework::Error;
    async fn process(&self, input: ListSudoMethods) -> Result<Vec<SudoMethod>, framework::Error> {
        let has_totp = self
            .db
            .process(FindTotpByUserId {
                user_id: input.user_id,
            })
            .await?
            .is_some();
        let mut methods = Vec::new();
        if has_totp {
            methods.push(SudoMethod::Totp);
        } else {
            methods.push(SudoMethod::Email);
        }
        Ok(methods)
    }
}

#[derive(Debug, Clone)]
pub struct EnterSudoMode {
    pub user_id: Uuid,
}

impl Processor<EnterSudoMode> for MfaService {
    type Output = SudoToken;
    type Error = framework::Error;
    async fn process(&self, input: EnterSudoMode) -> Result<SudoToken, framework::Error> {
        let mut redis = self.config_store.clone();
        let config = find_config_from_redis::<AuthConfig>(&mut redis).await?;
        let ttl: std::time::Duration = config.sudo_token_ttl.try_into().map_err(|e| {
            framework::Error::BusinessPanic(anyhow::anyhow!("Invalid sudo token ttl: {e}"))
        })?;
        let token = SudoToken {
            token: rand::random(),
            user_id: input.user_id,
        };
        token.write_with_ttl(&mut redis, ttl).await?;
        Ok(token)
    }
}

#[derive(Debug, Clone)]
pub struct VerifySudoToken {
    pub user_id: Uuid,
    pub token: [u8; 16],
}

impl Processor<VerifySudoToken> for MfaService {
    type Output = bool;
    type Error = framework::Error;
    async fn process(&self, input: VerifySudoToken) -> Result<bool, framework::Error> {
        let mut redis = self.config_store.clone();
        let token = SudoToken::read(&mut redis, SudoTokenKey(input.token)).await?;
        Ok(matches!(token, Some(token) if token.user_id == input.user_id))
    }
}

#[derive(Debug, Clone)]
pub enum SudoVerificationMethod {
    Totp(u32),
    EmailOtp(String),
}

impl SudoVerificationMethod {
    pub fn name(&self) -> &'static str {
        match self {
            SudoVerificationMethod::Totp(_) => "totp",
            SudoVerificationMethod::EmailOtp(_) => "email_otp",
        }
    }
}

#[derive(Debug, Clone)]
pub struct VerifyAndEnterSudo {
    pub user_id: Uuid,
    pub method: SudoVerificationMethod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyAndEnterSudoResult {
    Success([u8; 16]),
    InvalidCredential,
    MethodNotAllowed,
}

impl Processor<VerifyAndEnterSudo> for MfaService {
    type Output = VerifyAndEnterSudoResult;
    type Error = framework::Error;
    #[instrument(skip_all, err, name="VerifyAndEnterSudo", fields(
        user_id = %input.user_id,
        method = %input.method.name(),
    ))]
    async fn process(
        &self,
        input: VerifyAndEnterSudo,
    ) -> Result<VerifyAndEnterSudoResult, framework::Error> {
        let allowed_methods = self
            .process(ListSudoMethods {
                user_id: input.user_id,
            })
            .await?;
        let method_allowed = match &input.method {
            SudoVerificationMethod::Totp(_) => allowed_methods.contains(&SudoMethod::Totp),
            SudoVerificationMethod::EmailOtp(_) => allowed_methods.contains(&SudoMethod::Email),
        };
        if !method_allowed {
            return Ok(VerifyAndEnterSudoResult::MethodNotAllowed);
        }
        let verified = match input.method {
            SudoVerificationMethod::Totp(code) => {
                let Some(totp) = self
                    .db
                    .process(FindTotpByUserId {
                        user_id: input.user_id,
                    })
                    .await?
                else {
                    return Ok(VerifyAndEnterSudoResult::InvalidCredential);
                };
                verify_totp(totp.secret, code)
            } // SudoVerificationMethod::Totp(code) => { ... }
            SudoVerificationMethod::EmailOtp(code) => {
                let Some(user) = self
                    .db
                    .process(FindUserAccountById { id: input.user_id })
                    .await?
                else {
                    return Ok(VerifyAndEnterSudoResult::InvalidCredential);
                };
                let email = user.email;
                let Some(_otp) = self
                    .db
                    .process(FindValidEmailOtp {
                        email,
                        user_id: input.user_id,
                        usage: EmailOtpUsage::SudoMode,
                        otp_code: code,
                    })
                    .await?
                else {
                    return Ok(VerifyAndEnterSudoResult::InvalidCredential);
                };
                true
            } // SudoVerificationMethod::EmailOtp(code) => { ... }
        }; // let verified = match input.method { ... }
        if !verified {
            return Ok(VerifyAndEnterSudoResult::InvalidCredential);
        }
        let token = self
            .process(EnterSudoMode {
                user_id: input.user_id,
            })
            .await?;
        Ok(VerifyAndEnterSudoResult::Success(token.token))
    }
}

#[derive(Debug, Clone)]
pub struct VerifyMfaLogin {
    pub mfa_token: [u8; 32],
    pub code: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyMfaLoginResult {
    Success(SessionId),
    InvalidToken,
    InvalidCode,
    NoNeed,
}

impl Processor<VerifyMfaLogin> for MfaService {
    type Output = VerifyMfaLoginResult;
    type Error = framework::Error;
    async fn process(
        &self,
        input: VerifyMfaLogin,
    ) -> Result<VerifyMfaLoginResult, framework::Error> {
        let mut redis = self.redis.clone();

        // 1. Atomically read and delete the MFA login token from Redis using GETDEL
        // This prevents race conditions where concurrent requests could use the same token
        let key = MfaLoginTokenKey(input.mfa_token);
        let data: Option<Vec<u8>> = redis.get_del(key).await?;
        let Some(bytes) = data else {
            return Ok(VerifyMfaLoginResult::InvalidToken);
        };
        let mfa_login_token = MfaLoginToken::from_bytes(&bytes)
            .map_err(|e| framework::Error::DeserializeError(e.into()))?;

        // 2. Verify the TOTP code
        let Some(totp) = self
            .db
            .process(FindTotpByUserId {
                user_id: mfa_login_token.user_id,
            })
            .await?
        else {
            return Ok(VerifyMfaLoginResult::NoNeed);
        };

        if !verify_totp(totp.secret, input.code) {
            return Ok(VerifyMfaLoginResult::InvalidCode);
        }

        let session_id = self
            .session_service
            .process(CreateSession {
                user_id: mfa_login_token.user_id,
            })
            .await?;

        Ok(VerifyMfaLoginResult::Success(session_id))
    }
}

fn verify_totp(secret: Vec<u8>, code: u32) -> bool {
    let Ok(rfc6238) = totp_rs::Rfc6238::with_defaults(secret) else {
        return false;
    };
    let Ok(totp) = totp_rs::TOTP::from_rfc6238(rfc6238) else {
        return false;
    };
    let Ok(current) = totp.generate_current() else {
        return false;
    };
    current == code.to_string()
}
