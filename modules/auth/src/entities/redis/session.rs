use framework::redis::{KeyValue, KeyValueRead, KeyValueWrite, RedisKey};
use kanau::{RkyvMessageDe, RkyvMessageSer};
use uuid::Uuid;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    RkyvMessageSer,
    RkyvMessageDe,
)]
pub struct Session {
    pub id: SessionId,
    pub user_id: Uuid,
    pub terminated: bool,
    pub last_refreshed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
    pub fn to_ascii_string(&self) -> String {
        self.0.to_string()
    }
    pub fn try_from_ascii_string(s: &str) -> Result<Self, framework::Error> {
        let uuid = Uuid::parse_str(s).map_err(|_| framework::Error::InvalidInput)?;
        Ok(Self(uuid))
    }
}

impl redis::ToSingleRedisArg for SessionId {}

impl redis::ToRedisArgs for SessionId {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let key: RedisKey = RedisKey::from(format!("session:{}", self.0));
        key.write_redis_args(out);
    }
}

impl KeyValue for Session {
    type Key = SessionId;
    type Value = Self;

    fn key(&self) -> Self::Key {
        self.id
    }

    fn value(&self) -> Self::Value {
        self.clone()
    }

    fn into_value(self) -> Self::Value {
        self
    }

    fn new(key: Self::Key, mut value: Self::Value) -> Self {
        value.id = key;
        value
    }
}

impl KeyValueRead for Session {}
impl KeyValueWrite for Session {}
