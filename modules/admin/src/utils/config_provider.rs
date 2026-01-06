use redis::AsyncCommands;
use sqlx::PgPool;
use tracing::instrument;
use framework::redis::RedisConnection;

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
    let result: T = sqlx::query_as!(
        ApplicationConfig,
        "SELECT * FROM \"application__config\" WHERE key = $1",
        T::KEY
    )
    .fetch_optional(db)
    .await?
    .map(|result| serde_json::from_value(result.content))
    .transpose()
    .ok()
    .flatten()
    .unwrap_or_default();
    Ok(result)
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
    let key = format!("config:{}", T::KEY);
    let data: Option<Vec<u8>> = redis.get(key).await?;
    let cfg = data
        .and_then(|d| serde_json::from_slice(&d).ok())
        .unwrap_or_default();
    Ok(cfg)
}

#[instrument(
    skip_all,
    err,
    fields(
        config = std::any::type_name::<T>()
    )
)]
pub async fn refresh_config_cache<T: ConfigJson>(
    db: PgPool,
    mut redis: RedisConnection,
) -> Result<(), framework::Error> {
    let cfg = find_config_from_db::<T>(&db).await?;
    let key = format!("config:{}", T::KEY);
    let data = serde_json::to_vec(&cfg).map_err(|e| framework::Error::SerializeError(e.into()))?;
    let _: () = redis.set(key, data).await?;
    Ok(())
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
    let content = serde_json::to_value(config).map_err(|e| framework::Error::SerializeError(e.into()))?;
    sqlx::query!(
        r#"INSERT INTO application__config (key, content) VALUES ($1, $2)
         ON CONFLICT (key) DO UPDATE SET content = EXCLUDED.content"#,
        T::KEY,
        content
    )
        .execute(db)
        .await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct ConfigCronExecutor {
    pub db: PgPool,
    pub redis: RedisConnection,
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
