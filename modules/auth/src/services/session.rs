use framework::rabbitmq::AmqpPool;
use framework::redis::RedisConnection;

#[derive(Clone)]
pub struct SessionService {
    pub redis: RedisConnection,
    pub config_store: RedisConnection,
    pub mq: AmqpPool,
}
