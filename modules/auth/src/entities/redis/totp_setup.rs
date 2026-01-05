use kanau::{RkyvMessageDe, RkyvMessageSer};
use uuid::Uuid;

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
pub struct PendingTotpSetup {
    pub user_id: PendingTotpSetupKey,
    pub secret: Box<[u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
/// Wrap the user ID as the key for [PendingTotpSetup] in Redis and add the proper namespacing.
pub struct PendingTotpSetupKey(pub Uuid);

impl redis::ToSingleRedisArg for PendingTotpSetupKey {}

impl redis::ToRedisArgs for PendingTotpSetupKey {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let key: framework::redis::RedisKey =
            framework::redis::RedisKey::from(format!("totp_setup:{}", self.0));
        key.write_redis_args(out);
    }
}

impl core::fmt::Debug for PendingTotpSetup {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PendingTotpSetup")
            .field("user_id", &self.user_id)
            .field("secret", &"[redacted]")
            .finish()
    }
}

impl framework::redis::KeyValue for PendingTotpSetup {
    type Key = PendingTotpSetupKey;
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

impl framework::redis::KeyValueRead for PendingTotpSetup {}
impl framework::redis::KeyValueWrite for PendingTotpSetup {}
