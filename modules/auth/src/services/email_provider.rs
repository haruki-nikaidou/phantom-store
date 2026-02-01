use crate::config::AuthConfig;
use crate::entities::db::email_otp::{
    CheckEmailFrequency, CreateEmailOtp, EmailOtp, EmailOtpUsage, FindValidEmailOtp,
    MarkEmailOtpAsUsed,
};
use crate::entities::db::user_account::{FindUserAccountByEmail, FindUserAccountById, RegisterPasswordlessUserAccount, UpdateUserEmail};
use crate::entities::db::user_password::{
    DeleteUserPasswordByUserId, FindUserPasswordByEmail, FindUserPasswordByUserId,
    RegisterUserWithPassword, UpdateUserPassword,
};
use crate::entities::redis::session::SessionId;
use crate::events::account::{RegisterMethod, UserRegisterEvent};
use crate::events::email::OtpEmailSendCall;
use crate::services::mfa::{CheckMfaEnabled, CreateLoginMfaSession, MfaService, VerifySudoToken};
use crate::services::session::{CreateSession, SessionService, TerminateAllUserSessions};
use crate::utils::password::{hash_password, verify_password};
use admin::utils::config_provider::find_config_from_redis;
use framework::rabbitmq::{AmqpMessageSend, AmqpPool};
use framework::redis::RedisConnection;
use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use uuid::Uuid;

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
struct SendEmailOtp {
    pub email: String,
    pub usage: EmailOtpUsage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SendEmailOtpResult {
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

#[derive(Debug, Clone)]
pub struct SendRegisterEmail {
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendRegisterEmailResult {
    Sent,
    InvalidEmailAddress,
    DuplicatedEmail,
    RateLimited,
}

impl Processor<SendRegisterEmail> for EmailProviderService {
    type Output = SendRegisterEmailResult;
    type Error = framework::Error;
    async fn process(
        &self,
        input: SendRegisterEmail,
    ) -> Result<SendRegisterEmailResult, framework::Error> {
        // check if the email is already used
        let email_exists = self
            .db
            .process(FindUserAccountByEmail {
                email: input.email.clone(),
            })
            .await?
            .is_some();

        if email_exists {
            return Ok(SendRegisterEmailResult::DuplicatedEmail);
        }

        // send the email
        let send_result = self
            .process(SendEmailOtp {
                email: input.email.clone(),
                usage: EmailOtpUsage::Login,
            })
            .await?;
        match send_result {
            SendEmailOtpResult::Sent => Ok(SendRegisterEmailResult::Sent),
            SendEmailOtpResult::InvalidEmailAddress => {
                Ok(SendRegisterEmailResult::InvalidEmailAddress)
            }
            SendEmailOtpResult::RateLimited => Ok(SendRegisterEmailResult::RateLimited),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegisterUser {
    pub email: String,
    pub otp: String,
    pub password: Option<String>,
    pub name: Option<String>,
    pub auto_login: bool,
}

#[derive(Debug, Clone)]
pub enum RouteRegisterUserResult {
    WithPassword(RegisterWithPassword),
    Passwordless(RegisterPasswordless),
    InvalidOtp,
    DuplicatedEmail,
}

#[derive(Debug, Clone)]
pub struct RegisterWithPassword {
    pub password_hash: String,
    pub name: Option<String>,
    pub email: String,
    pub auto_login: bool,
}

#[derive(Debug, Clone)]
pub struct RegisterPasswordless {
    pub name: Option<String>,
    pub email: String,
    pub auto_login: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterUserResult {
    Registered,
    RegisteredWithSession(SessionId),
}

impl Processor<RegisterUser> for EmailProviderService {
    type Output = RouteRegisterUserResult;
    type Error = framework::Error;
    async fn process(
        &self,
        input: RegisterUser,
    ) -> Result<RouteRegisterUserResult, framework::Error> {
        // verify the otp
        let Some(otp) = self
            .process(VerifyEmailOtp {
                user_id: Uuid::nil(),
                email: input.email.clone(),
                otp: input.otp.clone(),
                usage: EmailOtpUsage::Login,
            })
            .await?
        else {
            return Ok(RouteRegisterUserResult::InvalidOtp);
        };

        // check if the email is already used
        let email_exists = self
            .db
            .process(FindUserAccountByEmail {
                email: input.email.clone(),
            })
            .await?
            .is_some();
        if email_exists {
            return Ok(RouteRegisterUserResult::DuplicatedEmail);
        }

        if let Some(password) = input.password {
            let password_hash =
                hash_password(&password).map_err(|_| framework::Error::InvalidInput)?;
            self.db.process(MarkEmailOtpAsUsed { id: otp.id }).await?;
            Ok(RouteRegisterUserResult::WithPassword(
                RegisterWithPassword {
                    password_hash,
                    name: input.name,
                    email: input.email,
                    auto_login: input.auto_login,
                },
            ))
        } else {
            self.db.process(MarkEmailOtpAsUsed { id: otp.id }).await?;
            Ok(RouteRegisterUserResult::Passwordless(
                RegisterPasswordless {
                    name: input.name,
                    email: input.email,
                    auto_login: input.auto_login,
                },
            ))
        }
    }
}

impl Processor<RegisterWithPassword> for EmailProviderService {
    type Output = RegisterUserResult;
    type Error = framework::Error;
    async fn process(
        &self,
        input: RegisterWithPassword,
    ) -> Result<RegisterUserResult, framework::Error> {
        let user_account = self
            .db
            .process(RegisterUserWithPassword {
                email: input.email.clone(),
                name: input.name.clone(),
                password_hash: input.password_hash.clone(),
            })
            .await?;
        UserRegisterEvent {
            user_id: user_account.user_id,
            registered_at: user_account.created_at.assume_utc().unix_timestamp() as u64,
            register_method: RegisterMethod::EmailAccount {
                user_id: user_account.user_id,
                has_password: true,
            },
            register_with_order_creation: false,
        }
        .send(&self.mq)
        .await?;
        if input.auto_login {
            let session_id = self
                .session_service
                .process(CreateSession {
                    user_id: user_account.user_id,
                })
                .await?;
            Ok(RegisterUserResult::RegisteredWithSession(session_id))
        } else {
            Ok(RegisterUserResult::Registered)
        }
    }
}

impl Processor<RegisterPasswordless> for EmailProviderService {
    type Output = RegisterUserResult;
    type Error = framework::Error;
    async fn process(
        &self,
        input: RegisterPasswordless,
    ) -> Result<RegisterUserResult, framework::Error> {
        let user_account = self
            .db
            .process(RegisterPasswordlessUserAccount {
                email: input.email.clone(),
                name: input.name.clone(),
            })
            .await?;
        UserRegisterEvent {
            user_id: user_account.id,
            registered_at: user_account.created_at.assume_utc().unix_timestamp() as u64,
            register_method: RegisterMethod::EmailAccount {
                user_id: user_account.id,
                has_password: false,
            },
            register_with_order_creation: false,
        }
        .send(&self.mq)
        .await?;
        if input.auto_login {
            let session_id = self
                .session_service
                .process(CreateSession {
                    user_id: user_account.id,
                })
                .await?;
            Ok(RegisterUserResult::RegisteredWithSession(session_id))
        } else {
            Ok(RegisterUserResult::Registered)
        }
    }
}

#[derive(Debug, Clone)]
pub struct SendPasswordResetEmail {
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendPasswordResetEmailResult {
    /// The email is sent or the email address does not exist.
    /// The user should not know if the email address exists.
    MaybeSent,

    /// The email address format is invalid.
    InvalidEmailAddress,

    /// The email is rate limited.
    RateLimited,
}

impl Processor<SendPasswordResetEmail> for EmailProviderService {
    type Output = SendPasswordResetEmailResult;
    type Error = framework::Error;
    async fn process(
        &self,
        input: SendPasswordResetEmail,
    ) -> Result<SendPasswordResetEmailResult, framework::Error> {
        let config = find_config_from_redis::<AuthConfig>(&mut self.config_store.clone()).await?;
        let account_exists = self
            .db
            .process(FindUserAccountByEmail {
                email: input.email.clone(),
            })
            .await?
            .is_some();
        let email_allowed = config.email_provider.domain.check_addr(&input.email);
        if !account_exists {
            return if email_allowed {
                Ok(SendPasswordResetEmailResult::MaybeSent)
            } else {
                Ok(SendPasswordResetEmailResult::InvalidEmailAddress)
            };
        }
        self.process(SendEmailOtp {
            email: input.email.clone(),
            usage: EmailOtpUsage::PasswordReset,
        })
        .await?;
        Ok(SendPasswordResetEmailResult::MaybeSent)
    }
}

#[derive(Debug, Clone)]
pub struct ResetPassword {
    pub email: String,
    pub otp: String,
    pub new_password: String,
    pub auto_login: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResetPasswordResult {
    Success,
    SuccessWithSession(SessionId),
    InvalidOtp,
    AccountNotFound,
}

impl Processor<ResetPassword> for EmailProviderService {
    type Output = ResetPasswordResult;
    type Error = framework::Error;
    async fn process(&self, input: ResetPassword) -> Result<ResetPasswordResult, framework::Error> {
        let Some(_) = self
            .process(VerifyEmailOtp {
                user_id: Uuid::nil(),
                email: input.email.clone(),
                otp: input.otp.clone(),
                usage: EmailOtpUsage::PasswordReset,
            })
            .await?
        else {
            return Ok(ResetPasswordResult::InvalidOtp);
        };
        let Some(user) = self
            .db
            .process(FindUserAccountByEmail {
                email: input.email.clone(),
            })
            .await?
        else {
            return Ok(ResetPasswordResult::AccountNotFound);
        };
        let password_hash =
            hash_password(&input.new_password).map_err(|_| framework::Error::InvalidInput)?;
        self.db
            .process(UpdateUserPassword {
                user_id: user.id,
                password_hash,
            })
            .await?;
        self.session_service
            .process(TerminateAllUserSessions { user_id: user.id })
            .await?;
        if input.auto_login {
            let session_id = self
                .session_service
                .process(CreateSession { user_id: user.id })
                .await?;
            Ok(ResetPasswordResult::SuccessWithSession(session_id))
        } else {
            Ok(ResetPasswordResult::Success)
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserChangePassword {
    pub user_id: Uuid,
    pub sudo_token: [u8; 16],
    pub new_password: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangePasswordResult {
    Success,
    SudoFailed,
    NotFound,
}

impl Processor<UserChangePassword> for EmailProviderService {
    type Output = ChangePasswordResult;
    type Error = framework::Error;
    async fn process(
        &self,
        input: UserChangePassword,
    ) -> Result<ChangePasswordResult, framework::Error> {
        let verified = self
            .mfa_service
            .process(VerifySudoToken {
                user_id: input.user_id,
                token: input.sudo_token,
            })
            .await?;
        if !verified {
            return Ok(ChangePasswordResult::SudoFailed);
        }
        let Some(_) = self
            .db
            .process(FindUserAccountById { id: input.user_id })
            .await?
        else {
            return Ok(ChangePasswordResult::NotFound);
        };
        let password_hash =
            hash_password(&input.new_password).map_err(|_| framework::Error::InvalidInput)?;
        self.db
            .process(UpdateUserPassword {
                user_id: input.user_id,
                password_hash,
            })
            .await?;
        Ok(ChangePasswordResult::Success)
    }
}

#[derive(Debug, Clone)]
pub struct UserRemovePassword {
    pub user_id: Uuid,
    pub sudo_token: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemovePasswordResult {
    Success,
    SudoFailed,
    AlreadyRemoved,
    NotFound,
}

impl Processor<UserRemovePassword> for EmailProviderService {
    type Output = RemovePasswordResult;
    type Error = framework::Error;
    async fn process(
        &self,
        input: UserRemovePassword,
    ) -> Result<RemovePasswordResult, framework::Error> {
        let verified = self
            .mfa_service
            .process(VerifySudoToken {
                user_id: input.user_id,
                token: input.sudo_token,
            })
            .await?;
        if !verified {
            return Ok(RemovePasswordResult::SudoFailed);
        }
        let Some(_) = self
            .db
            .process(FindUserAccountById { id: input.user_id })
            .await?
        else {
            return Ok(RemovePasswordResult::NotFound);
        };
        let Some(password) = self
            .db
            .process(FindUserPasswordByUserId {
                user_id: input.user_id,
            })
            .await?
        else {
            return Ok(RemovePasswordResult::AlreadyRemoved);
        };
        self.db
            .process(DeleteUserPasswordByUserId {
                user_id: password.user_id,
            })
            .await?;
        Ok(RemovePasswordResult::Success)
    }
}

#[derive(Debug, Clone)]
pub struct UserChangeEmailAddress {
    pub user_id: Uuid,
    pub sudo_token: [u8; 16],
    pub new_email: String,
    pub otp: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeEmailAddressResult {
    Success,
    SudoFailed,
    InvalidEmail,
    InvalidOtp,
    NotFound,
}

impl Processor<UserChangeEmailAddress> for EmailProviderService {
    type Output = ChangeEmailAddressResult;
    type Error = framework::Error;
    async fn process(
        &self,
        input: UserChangeEmailAddress,
    ) -> Result<ChangeEmailAddressResult, framework::Error> {
        let verified = self
            .mfa_service
            .process(VerifySudoToken {
                user_id: input.user_id,
                token: input.sudo_token,
            })
            .await?;
        if !verified {
            return Ok(ChangeEmailAddressResult::SudoFailed);
        }
        let Some(otp) = self
            .process(VerifyEmailOtp {
                user_id: input.user_id,
                email: input.new_email.clone(),
                otp: input.otp.clone(),
                usage: EmailOtpUsage::ChangeEmailAddress,
            })
            .await?
        else {
            return Ok(ChangeEmailAddressResult::InvalidOtp);
        };
        let Some(_) = self
            .db
            .process(FindUserAccountById { id: input.user_id })
            .await?
        else {
            return Ok(ChangeEmailAddressResult::NotFound);
        };
        let config = find_config_from_redis::<AuthConfig>(&mut self.config_store.clone()).await?;
        if !config.email_provider.domain.check_addr(&input.new_email) {
            return Ok(ChangeEmailAddressResult::InvalidEmail);
        }
        let account_exists = self
            .db
            .process(FindUserAccountByEmail {
                email: input.new_email.clone(),
            })
            .await?
            .is_some();
        if account_exists {
            return Ok(ChangeEmailAddressResult::InvalidEmail);
        }
        self.db
            .process(UpdateUserEmail {
                id: input.user_id,
                email: input.new_email.clone(),
            })
            .await?;
        Ok(ChangeEmailAddressResult::Success)
    }
}

#[derive(Debug, Clone)]
struct VerifyEmailOtp {
    pub user_id: Uuid,
    pub email: String,
    pub otp: String,
    pub usage: EmailOtpUsage,
}

impl Processor<VerifyEmailOtp> for EmailProviderService {
    type Output = Option<EmailOtp>;
    type Error = framework::Error;
    async fn process(&self, input: VerifyEmailOtp) -> Result<Option<EmailOtp>, framework::Error> {
        self.db
            .process(FindValidEmailOtp {
                user_id: input.user_id,
                email: input.email,
                usage: input.usage,
                otp_code: input.otp,
            })
            .await
            .map_err(Into::into)
    }
}
