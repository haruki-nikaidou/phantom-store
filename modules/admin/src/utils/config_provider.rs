use tracing::instrument;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ApplicationConfig {
    #[allow(unused)]
    pub id: i32,
    #[allow(unused)]
    pub key: String,
    pub content: serde_json::Value,
}

pub trait ConfigJson: Default + for<'de> serde::Deserialize<'de> + serde::Serialize {
    const KEY: &'static str;
}

#[instrument(
    skip_all,
    err,
    fields(
        config = std::any::type_name::<T>(),
        config_key = T::KEY
    )
)]
pub async fn find_config_from_db<T: ConfigJson>(db: impl sqlx::PgExecutor<'_>) -> sqlx::Result<T> {
    todo!()
}

#[instrument(
    skip_all,
    err,
    fields(
        config = std::any::type_name::<T>(),
        config_key = T::KEY
    )
)]
pub async fn find_config_from_redis<T: ConfigJson>(
    redis: &mut impl redis::AsyncCommands,
) -> redis::RedisResult<T> {
    todo!()
}

#[instrument(
    skip_all,
    err,
    fields(
        config = std::any::type_name::<T>()
    )
)]
pub async fn refresh_config_cache<T: ConfigJson>(
    db: impl sqlx::PgExecutor<'_>,
    redis: &mut impl redis::AsyncCommands,
) -> Result<(), framework::Error> {
    todo!()
}

#[instrument(
    skip(db),
    err,
    fields(
        config_key = T::KEY
    )
)]
pub async fn insert_config_into_db<T: ConfigJson + std::fmt::Debug>(
    db: impl sqlx::PgExecutor<'_>,
    config: &T,
) -> Result<(), framework::Error> {
    todo!()
}

#[derive(Debug, Clone)]
pub struct ConfigCronExecutor {
    pub db: sqlx::PgPool,
    pub redis: framework::redis::RedisConnection,
}

#[macro_export]
macro_rules! refresh_configs {
    ($executor:expr, [$($config:ty),+]) => {{
        let ConfigCronExecutor { db, redis } = $executor;
        let mut join_set = tokio::task::JoinSet::<Result<(), helium_framework::error::Error>>::new();
        $(
            join_set.spawn($crate::config_provider::refresh_config::<$config>(db.clone(), redis.clone()));
        )+
        join_set.join_all().await
    }};
}
