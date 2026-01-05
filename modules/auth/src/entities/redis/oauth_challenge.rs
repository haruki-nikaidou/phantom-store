use crate::utils::oauth::providers::OAuthProviderName;
use kanau::{RkyvMessageDe, RkyvMessageSer};
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum OAuthAction {
    Login,
    Bind { user_id: Uuid },
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
pub struct OAuthChallenge {
    pub state: OAuthChallengeKey,
    pub provider_name: OAuthProviderName,
    pub action: OAuthAction,
    pub redirect_uri: String,
    pub pkce_verifier: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct OAuthChallengeKey(pub [u8; 32]);

impl core::fmt::Display for OAuthChallengeKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl std::str::FromStr for OAuthChallengeKey {
    type Err = framework::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex_bytes = hex::decode(s).map_err(|_| framework::Error::InvalidInput)?;
        let array = hex_bytes
            .try_into()
            .map_err(|_| framework::Error::InvalidInput)?;
        Ok(Self(array))
    }
}

impl OAuthChallengeKey {
    pub fn generate() -> Self {
        Self(rand::random())
    }
}

impl redis::ToSingleRedisArg for OAuthChallengeKey {}

impl redis::ToRedisArgs for OAuthChallengeKey {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let string = format!("oauth_challenge:{}", hex::encode(self.0));
        let key: framework::redis::RedisKey = framework::redis::RedisKey::from(string);
        key.write_redis_args(out);
    }
}

impl core::fmt::Debug for OAuthChallenge {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OAuthLoginChallenge")
            .field("state", &"[redacted]")
            .field("provider_name", &self.provider_name)
            .field("action", &self.action)
            .finish()
    }
}

impl framework::redis::KeyValue for OAuthChallenge {
    type Key = OAuthChallengeKey;
    type Value = Self;

    fn key(&self) -> Self::Key {
        self.state
    }

    fn value(&self) -> Self::Value {
        self.clone()
    }

    fn into_value(self) -> Self::Value {
        self
    }

    fn new(key: Self::Key, mut value: Self::Value) -> Self {
        value.state = key;
        value
    }
}

impl framework::redis::KeyValueRead for OAuthChallenge {}
impl framework::redis::KeyValueWrite for OAuthChallenge {}

impl OAuthChallenge {
    /// Atomically read and delete the challenge from Redis using GETDEL.
    /// This prevents race conditions where the same challenge could be used multiple times.
    #[instrument(skip_all, name = "OAuthChallenge::read_and_delete", err)]
    pub async fn read_and_delete(
        conn: &mut framework::redis::RedisConnection,
        key: OAuthChallengeKey,
    ) -> Result<Option<Self>, framework::Error> {
        use kanau::message::MessageDe;
        use redis::AsyncCommands;

        let data: Option<Vec<u8>> = conn.get_del(key).await?;
        if let Some(bytes) = data {
            let val = <Self as MessageDe>::from_bytes(&bytes)
                .map_err(|e| framework::Error::DeserializeError(e.into()))?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }
}
