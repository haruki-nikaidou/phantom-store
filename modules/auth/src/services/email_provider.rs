use crate::entities::db::user_account::FindUserAccountByEmail;
use crate::entities::db::user_password::{FindUserPasswordByEmail, FindUserPasswordByUserId};
use crate::entities::redis::session::SessionId;
use crate::services::mfa::{CheckMfaEnabled, CreateLoginMfaSession, MfaService};
use crate::services::session::{CreateSession, SessionService};
use crate::utils::password::verify_password;
use framework::rabbitmq::AmqpPool;
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
