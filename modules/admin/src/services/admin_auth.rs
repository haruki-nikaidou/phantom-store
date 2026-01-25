use crate::entities::admin_session::AdminSessionId;
use framework::redis::RedisConnection;
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
        todo!()
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
        todo!()
    }
}
