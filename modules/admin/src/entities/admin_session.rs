use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct AdminSessionId(pub [u8; 32]);

impl AdminSessionId {
    pub fn generate() -> Self {
        Self(rand::random())
    }

    pub fn to_ascii_string(&self) -> String {
        hex::encode(self.0)
    }

    pub fn try_from_ascii_string(s: &str) -> Result<Self, framework::Error> {
        let bytes = hex::decode(s).map_err(|_| framework::Error::InvalidInput)?;
        let array = bytes
            .try_into()
            .map_err(|_| framework::Error::InvalidInput)?;
        Ok(Self(array))
    }
}

impl redis::ToSingleRedisArg for AdminSessionId {}

impl redis::ToRedisArgs for AdminSessionId {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let key: framework::redis::RedisKey =
            framework::redis::RedisKey::from(format!("admin_session:{}", hex::encode(self.0)));
        key.write_redis_args(out);
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    kanau::RkyvMessageSer,
    kanau::RkyvMessageDe,
)]
pub struct AdminSession {
    pub id: AdminSessionId,
    pub admin_id: Uuid,
}

impl framework::redis::KeyValue for AdminSession {
    type Key = AdminSessionId;
    type Value = Self;

    fn key(&self) -> Self::Key {
        self.id
    }

    fn value(&self) -> Self::Value {
        *self
    }

    fn into_value(self) -> Self::Value {
        self
    }

    fn new(key: Self::Key, mut value: Self::Value) -> Self {
        value.id = key;
        value
    }
}

impl framework::redis::KeyValueRead for AdminSession {}

impl framework::redis::KeyValueWrite for AdminSession {}
