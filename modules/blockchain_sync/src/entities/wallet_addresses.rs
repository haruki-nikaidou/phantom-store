use crate::utils::supported_tokens::{FlattenSupportedBlockchains, StableCoinName};

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct MerchantWalletAddress {
    pub id: i64,
    pub address: String,
    pub chain: FlattenSupportedBlockchains,
    pub enabled_stable_coins: Vec<StableCoinName>,
    pub active: bool,
}