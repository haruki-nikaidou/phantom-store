use crate::entities::admin_account::FindAdminByEmail;
use crate::entities::admin_session::{AdminSession, AdminSessionId};
use crate::utils::password::verify_password;
use framework::redis::{KeyValue, KeyValueWrite, RedisConnection};
use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use tracing::instrument;

#[derive(Clone)]
pub struct AdminAuthService {
    pub db: DatabaseProcessor,
    pub redis: RedisConnection,
}

#[derive(Debug, Clone)]
pub struct AdminLogin {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminLoginResult {
    Success(AdminSessionId),
    WrongCredential,
}

impl Processor<AdminLogin> for AdminAuthService {
    type Output = AdminLoginResult;
    type Error = framework::Error;
    #[instrument(skip_all, err)]
    async fn process(&self, input: AdminLogin) -> Result<AdminLoginResult, framework::Error> {
        // Find admin by email
        let Some(admin) = self
            .db
            .process(FindAdminByEmail { email: input.email })
            .await?
        else {
            return Ok(AdminLoginResult::WrongCredential);
        };

        // Verify password
        if verify_password(&input.password, &admin.password_hash).is_err() {
            return Ok(AdminLoginResult::WrongCredential);
        }

        // Create session
        let session_id = AdminSessionId::generate();
        let session = AdminSession::new(
            session_id,
            AdminSession {
                id: session_id,
                admin_id: admin.id,
            },
        );

        // Store session in Redis
        session.write(&mut self.redis.clone()).await?;

        Ok(AdminLoginResult::Success(session_id))
    }
}

pub struct AdminLogout {
    pub session_id: AdminSessionId,
}

impl Processor<AdminLogout> for AdminAuthService {
    type Output = ();
    type Error = framework::Error;
    #[instrument(skip_all, err)]
    async fn process(&self, input: AdminLogout) -> Result<(), framework::Error> {
        // Delete session from Redis
        AdminSession::delete(&mut self.redis.clone(), input.session_id).await?;
        Ok(())
    }
}
