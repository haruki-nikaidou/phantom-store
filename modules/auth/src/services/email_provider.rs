use crate::config::AuthConfig;
use crate::entities::db::email_otp::{CheckEmailFrequency, CreateEmailOtp, EmailOtpUsage};
use crate::entities::db::user_account::FindUserAccountByEmail;
use crate::entities::db::user_password::{FindUserPasswordByEmail, FindUserPasswordByUserId};
use crate::entities::redis::session::SessionId;
use crate::events::email::OtpEmailSendCall;
use crate::services::mfa::{CheckMfaEnabled, CreateLoginMfaSession, MfaService};
use crate::services::session::{CreateSession, SessionService};
use crate::utils::password::verify_password;
use admin::utils::config_provider::find_config_from_redis;
use framework::rabbitmq::{AmqpMessageSend, AmqpPool};
use framework::redis::RedisConnection;
use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;

#[derive(Clone)]
pub struct EmailProviderService {
    pub db: DatabaseProcessor,
    pub config_store: RedisConnection,
    pub redis: RedisConnection,
    pub mq: AmqpPool,
    pub session_service: SessionService,
    pub mfa_service: MfaService,
}

#[derive(Debug, Clone)]
pub struct ListEmailLoginMethod {
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailLoginMethod {
    Otp,
    Password,
}

impl Processor<ListEmailLoginMethod> for EmailProviderService {
    type Output = Vec<EmailLoginMethod>;
    type Error = framework::Error;
    async fn process(
        &self,
        input: ListEmailLoginMethod,
    ) -> Result<Vec<EmailLoginMethod>, framework::Error> {
        let Some(account) = self
            .db
            .process(FindUserAccountByEmail { email: input.email })
            .await?
        else {
            return Ok(Vec::new());
        };
        let password_exists = self
            .db
            .process(FindUserPasswordByUserId {
                user_id: account.id,
            })
            .await?
            .is_some();
        if password_exists {
            Ok(vec![EmailLoginMethod::Password, EmailLoginMethod::Otp])
        } else {
            Ok(vec![EmailLoginMethod::Otp])
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoginUserWithPassword {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailLoginResult {
    Success(SessionId),
    WrongCredential,
    MethodNotAvailable,
    MfaRequired([u8; 32]),
}

impl Processor<LoginUserWithPassword> for EmailProviderService {
    type Output = EmailLoginResult;
    type Error = framework::Error;
    async fn process(
        &self,
        input: LoginUserWithPassword,
    ) -> Result<EmailLoginResult, framework::Error> {
        let Some(password) = self
            .db
            .process(FindUserPasswordByEmail { email: input.email })
            .await?
        else {
            // safety: this function should be called after routing the login method.
            // So, this branch should be unreachable.
            // To maintain constant time, a fake verification should be performed at the route level.
            return Ok(EmailLoginResult::MethodNotAvailable);
        };
        if verify_password(&input.password, &password.password_hash).is_err() {
            return Ok(EmailLoginResult::WrongCredential);
        };

        let mfa_enabled = self
            .mfa_service
            .process(CheckMfaEnabled {
                user_id: password.user_id,
            })
            .await?;

        if mfa_enabled {
            let token = self
                .mfa_service
                .process(CreateLoginMfaSession {
                    user_id: password.user_id,
                })
                .await?;
            Ok(EmailLoginResult::MfaRequired(token.token))
        } else {
            let session_id = self
                .session_service
                .process(CreateSession {
                    user_id: password.user_id,
                })
                .await?;
            Ok(EmailLoginResult::Success(session_id))
        }
    }
}

#[derive(Debug, Clone)]
pub struct SendEmailOtp {
    pub email: String,
    pub usage: EmailOtpUsage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendEmailOtpResult {
    Sent,
    InvalidEmailAddress,
    RateLimited,
}

impl Processor<SendEmailOtp> for EmailProviderService {
    type Output = SendEmailOtpResult;
    type Error = framework::Error;
    async fn process(&self, input: SendEmailOtp) -> Result<SendEmailOtpResult, framework::Error> {
        let config = find_config_from_redis::<AuthConfig>(&mut self.config_store.clone()).await?;
        let email_allowed = config.email_provider.domain.check_addr(&input.email);
        if !email_allowed {
            return Ok(SendEmailOtpResult::InvalidEmailAddress);
        }
        let now = framework::now_time();
        let before = now - config.email_provider.otp.resend_interval;
        let freq = self
            .db
            .process(CheckEmailFrequency {
                email: input.email.clone(),
                before,
            })
            .await?;
        if freq > 1 {
            return Ok(SendEmailOtpResult::RateLimited);
        }
        let expires_after = config.email_provider.otp.expire_after.as_seconds_f64();
        let expires_after = sqlx::postgres::types::PgInterval {
            months: 0,
            days: 0,
            microseconds: (expires_after * 1_000_000.0) as i64,
        };
        let otp = self
            .db
            .process(CreateEmailOtp {
                user_id: None,
                email: input.email.clone(),
                usage: input.usage,
                expires_after,
            })
            .await?;
        OtpEmailSendCall {
            email_address: input.email,
            otp_code: otp.otp_code,
            otp_usage: input.usage,
            expire_after: config
                .email_provider
                .otp
                .expire_after
                .try_into()
                .map_err(|e| {
                    framework::Error::BusinessPanic(anyhow::anyhow!("Invalid duration: {e}"))
                })?,
            sent_at: now.assume_utc().unix_timestamp() as u64,
        }
        .send(&self.mq)
        .await?;
        Ok(SendEmailOtpResult::Sent)
    }
}
