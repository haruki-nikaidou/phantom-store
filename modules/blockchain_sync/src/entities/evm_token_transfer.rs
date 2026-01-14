use crate::services::etherscan::EtherScanChain;
use crate::utils::supported_tokens::StableCoinName;

pub struct EvmTokenTransfer {
    pub id: i64,
    pub token_name: StableCoinName,
    pub chain: EtherScanChain,
    pub from_address: String,
    pub to_address: String,
    pub value: rust_decimal::Decimal,
    pub block_number: u64,
    pub block_timestamp: time::PrimitiveDateTime,
}