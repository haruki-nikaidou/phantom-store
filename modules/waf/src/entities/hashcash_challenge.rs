use framework::redis::{KeyValue, KeyValueRead, KeyValueWrite, RedisKey};
use kanau::{RkyvMessageDe, RkyvMessageSer};
use sha2::{Digest, Sha256};

#[derive(
    Debug,
    Clone,
    PartialEq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    RkyvMessageSer,
    RkyvMessageDe,
)]
pub struct HashcashChallenge {
    pub id: HashcashId,
    pub challenge: [u8; 32],
    pub difficulty: u32,
}

impl HashcashChallenge {
    pub fn generate(difficulty: u32) -> Self {
        Self {
            id: HashcashId(rand::random()),
            challenge: rand::random(),
            difficulty,
        }
    }

    fn count_leading_zero_bits(bytes: &[u8]) -> u32 {
        let mut count = 0u32;
        for b in bytes {
            if *b == 0 {
                count += 8;
            } else {
                count += b.leading_zeros();
                break;
            }
        }
        count
    }

    pub fn verify(&self, nonce: u64) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(self.challenge);
        hasher.update(nonce.to_be_bytes());
        let hash = hasher.finalize();
        Self::count_leading_zero_bits(&hash) >= self.difficulty
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct HashcashId(pub [u8; 16]);

impl From<HashcashId> for RedisKey {
    fn from(value: HashcashId) -> Self {
        let string = format!("hashcash:{}", hex::encode(value.0));
        Self::from(string)
    }
}

impl redis::ToSingleRedisArg for HashcashId {}

impl redis::ToRedisArgs for HashcashId {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let key: RedisKey = (*self).into();
        key.write_redis_args(out);
    }
}

impl KeyValue for HashcashChallenge {
    type Key = HashcashId;
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

impl KeyValueRead for HashcashChallenge {}
impl KeyValueWrite for HashcashChallenge {}
