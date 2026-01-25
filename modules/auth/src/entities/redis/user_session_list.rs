use framework::redis::{KeyValue, KeyValueRead, KeyValueWrite, RedisConnection, RedisKey};
use kanau::{RkyvMessageDe, RkyvMessageSer};
use redis::AsyncCommands;
use tracing::instrument;
use uuid::Uuid;

/// Represents the set of active session IDs for a user, stored as a Redis SET.
/// Note: The derive macros are required to satisfy trait bounds, but the actual
/// read/write operations use Redis SET commands (SADD, SREM, SMEMBERS) instead.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    RkyvMessageDe,
    RkyvMessageSer,
)]
pub struct UserSessions {
    pub user_id: UserSessionIndex,
    pub session_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct UserSessionIndex(pub Uuid);

impl From<Uuid> for UserSessionIndex {
    fn from(v: Uuid) -> Self {
        Self(v)
    }
}

impl redis::ToSingleRedisArg for UserSessionIndex {}

impl redis::ToRedisArgs for UserSessionIndex {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let key: RedisKey = RedisKey::from(format!("user_sessions_set:{}", self.0));
        key.write_redis_args(out);
    }
}

impl KeyValue for UserSessions {
    type Key = UserSessionIndex;
    type Value = Self;

    fn key(&self) -> Self::Key {
        self.user_id
    }

    fn value(&self) -> Self::Value {
        self.clone()
    }

    fn into_value(self) -> Self::Value {
        self
    }

    fn new(key: Self::Key, mut value: Self::Value) -> Self {
        value.user_id = key;
        value
    }
}

impl KeyValueRead for UserSessions {
    /// Read all session IDs from the Redis SET using SMEMBERS.
    #[instrument(skip_all, fields(value_type = std::any::type_name::<Self>()))]
    async fn read(
        conn: &mut RedisConnection,
        key: Self::Key,
    ) -> Result<Option<Self::Value>, framework::Error> {
        let session_strings: Vec<String> = conn.smembers(key).await?;
        if session_strings.is_empty() {
            return Ok(None);
        }
        let session_ids: Vec<Uuid> = session_strings
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();
        Ok(Some(UserSessions {
            user_id: key,
            session_ids,
        }))
    }
}

impl KeyValueWrite for UserSessions {
    /// Write all session IDs to the Redis SET using SADD.
    /// This replaces the entire set with the provided session_ids.
    #[instrument(skip_all, fields(value_type = std::any::type_name::<Self>()))]
    async fn write_kv(
        conn: &mut RedisConnection,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), framework::Error> {
        // Delete existing set and add all current session IDs
        let _: () = conn.del(key).await?;
        if !value.session_ids.is_empty() {
            let session_strings: Vec<String> =
                value.session_ids.iter().map(|id| id.to_string()).collect();
            let _: () = conn.sadd(key, session_strings).await?;
        }
        Ok(())
    }

    /// Write all session IDs to the Redis SET with TTL.
    /// Note: Redis SETs don't support per-member TTL, so TTL applies to the entire set.
    #[instrument(skip_all, fields(value_type = std::any::type_name::<Self>()))]
    async fn write_kv_with_ttl(
        conn: &mut RedisConnection,
        key: Self::Key,
        value: Self::Value,
        ttl: std::time::Duration,
    ) -> Result<(), framework::Error> {
        // Delete existing set and add all current session IDs
        let _: () = conn.del(key).await?;
        if !value.session_ids.is_empty() {
            let session_strings: Vec<String> =
                value.session_ids.iter().map(|id| id.to_string()).collect();
            let _: () = conn.sadd(key, session_strings).await?;
        }
        // Set TTL on the entire set
        let _: () = conn.expire(key, ttl.as_secs() as i64).await?;
        Ok(())
    }
}

impl UserSessions {
    /// Add a session ID to the user's session set using SADD.
    #[instrument(skip_all, fields(user_id = %user_id.0, session_id = %session_id))]
    pub async fn add_session(
        conn: &mut RedisConnection,
        user_id: UserSessionIndex,
        session_id: Uuid,
    ) -> Result<(), framework::Error> {
        let _: () = conn.sadd(user_id, session_id.to_string()).await?;
        Ok(())
    }

    /// Remove a session ID from the user's session set using SREM.
    #[instrument(skip_all, fields(user_id = %user_id.0, session_id = %session_id))]
    pub async fn remove_session(
        conn: &mut RedisConnection,
        user_id: UserSessionIndex,
        session_id: Uuid,
    ) -> Result<(), framework::Error> {
        let _: () = conn.srem(user_id, session_id.to_string()).await?;
        Ok(())
    }

    /// Check if a session ID exists in the user's session set using SISMEMBER.
    #[instrument(skip_all, fields(user_id = %user_id.0, session_id = %session_id))]
    pub async fn has_session(
        conn: &mut RedisConnection,
        user_id: UserSessionIndex,
        session_id: Uuid,
    ) -> Result<bool, framework::Error> {
        let exists: bool = conn.sismember(user_id, session_id.to_string()).await?;
        Ok(exists)
    }

    /// Get the count of active sessions for a user using SCARD.
    #[instrument(skip_all, fields(user_id = %user_id.0))]
    pub async fn session_count(
        conn: &mut RedisConnection,
        user_id: UserSessionIndex,
    ) -> Result<usize, framework::Error> {
        let count: usize = conn.scard(user_id).await?;
        Ok(count)
    }
}
