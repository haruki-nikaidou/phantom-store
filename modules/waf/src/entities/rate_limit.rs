use framework::redis::{KeyValue, KeyValueRead, KeyValueWrite, RedisKey};
use kanau::{RkyvMessageDe, RkyvMessageSer};

#[derive(
    Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, RkyvMessageSer, RkyvMessageDe,
)]
pub struct RateLimitBucket {
    pub id: BucketId,
    pub tokens: f64,
    pub last_refill: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct BucketId {
    pub api_name: String,
    pub tenant: String
}

impl From<BucketId> for RedisKey {
    fn from(value: BucketId) -> Self {
        let string = format!("rate_limit:[{}]-{}", value.api_name, value.tenant);
        Self::from(string)
    }
}

impl redis::ToSingleRedisArg for BucketId {}

impl redis::ToRedisArgs for BucketId {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let key: RedisKey = self.clone().into();
        key.write_redis_args(out);
    }
}

impl KeyValue for RateLimitBucket {
    type Key = BucketId;
    type Value = Self;

    fn key(&self) -> Self::Key {
        self.id.clone()
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

impl KeyValueRead for RateLimitBucket {}
impl KeyValueWrite for RateLimitBucket {}
