use framework::rabbitmq::AmqpPool;
use framework::redis::RedisConnection;
use framework::sqlx::DatabaseProcessor;
use crate::services::mfa::MfaService;
use crate::services::session::SessionService;

#[derive(Clone)]
pub struct OAUthProviderService {
    pub db: DatabaseProcessor,
    pub config_store: RedisConnection,
    pub redis: RedisConnection,
    pub mq: AmqpPool,
    pub session_service: SessionService,
    pub mfa_service: MfaService,
}
