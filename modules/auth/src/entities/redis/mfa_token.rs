use framework::redis::RedisKey;
use uuid::Uuid;

/// Wrapper type for MFA login token key with proper namespacing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MfaLoginTokenKey(pub [u8; 32]);

impl redis::ToSingleRedisArg for MfaLoginTokenKey {}

impl redis::ToRedisArgs for MfaLoginTokenKey {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let hex = hex::encode(self.0);
        let string = format!("mfa_login:{}", hex);
        let key: RedisKey = RedisKey::from(string);
        key.write_redis_args(out);
    }
}

/// Token stored in Redis when a user has valid credentials but needs MFA verification.
/// Contains all necessary information to complete the login after TOTP verification.
#[derive(
    Clone,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    kanau::RkyvMessageDe,
    kanau::RkyvMessageSer,
)]
pub struct MfaLoginToken {
    /// Randomly generated 256-bit token
    pub token: [u8; 32],
    /// User ID attempting to log in
    pub user_id: Uuid,
}

impl core::fmt::Debug for MfaLoginToken {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MfaLoginToken")
            .field("token", &"[redacted]")
            .field("user_id", &self.user_id)
            .finish()
    }
}

impl framework::redis::KeyValue for MfaLoginToken {
    type Key = MfaLoginTokenKey;
    type Value = Self;

    fn key(&self) -> Self::Key {
        MfaLoginTokenKey(self.token)
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

impl framework::redis::KeyValueRead for MfaLoginToken {}
impl framework::redis::KeyValueWrite for MfaLoginToken {}
