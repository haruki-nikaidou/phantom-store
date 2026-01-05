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
        let key: RedisKey = RedisKey::from(format!("user_sessions_list:{}", self.0));
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

impl KeyValueRead for UserSessions {}
impl KeyValueWrite for UserSessions {}
