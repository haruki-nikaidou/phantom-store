use framework::redis::{KeyValue, KeyValueRead, KeyValueWrite, RedisKey};
use kanau::{RkyvMessageDe, RkyvMessageSer};
use uuid::Uuid;

/// Wrapper type for sudo token key with proper namespacing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SudoTokenKey(pub [u8; 16]);

impl redis::ToSingleRedisArg for SudoTokenKey {}

impl redis::ToRedisArgs for SudoTokenKey {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let key: RedisKey = RedisKey::from(format!("sudo:{}", hex::encode(self.0)));
        key.write_redis_args(out);
    }
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    RkyvMessageDe,
    RkyvMessageSer,
)]
pub struct SudoToken {
    pub token: [u8; 16],
    pub user_id: Uuid,
}

impl core::fmt::Debug for SudoToken {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SudoToken")
            .field("token", &"[redacted]")
            .field("user_id", &self.user_id)
            .finish()
    }
}

impl KeyValue for SudoToken {
    type Key = SudoTokenKey;
    type Value = Self;

    fn key(&self) -> Self::Key {
        SudoTokenKey(self.token)
    }

    fn value(&self) -> Self::Value {
        self.clone()
    }

    fn into_value(self) -> Self::Value {
        self
    }

    fn new(key: Self::Key, mut value: Self::Value) -> Self {
        value.token = key.0;
        value
    }
}

impl KeyValueRead for SudoToken {}
impl KeyValueWrite for SudoToken {}
