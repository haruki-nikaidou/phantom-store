use crate::utils::supported_tokens::FlattenSupportedBlockchains;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct CustomerAddresses {
    pub id: i64,
    pub user_id: Uuid,
    pub chain: FlattenSupportedBlockchains,
    pub address: String,
}
