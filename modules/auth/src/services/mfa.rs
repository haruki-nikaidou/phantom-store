use crate::services::session::SessionService;
use framework::rabbitmq::AmqpPool;
use framework::redis::RedisConnection;
use framework::sqlx::DatabaseProcessor;

#[derive(Clone)]
pub struct MfaService {
    pub db: DatabaseProcessor,
    pub config_store: RedisConnection,
    pub redis: RedisConnection,
    pub mq: AmqpPool,
    pub session_service: SessionService,
}
