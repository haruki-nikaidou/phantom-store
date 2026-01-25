use crate::services::etherscan::EtherScanChain;
use crate::utils::supported_tokens::StableCoinName;

pub struct Erc20StablecoinPendingDeposit {
    pub id: i64,
    pub token_name: StableCoinName,
    pub chain: EtherScanChain,
    pub user_address: Option<String>,
    pub wallet_address: String,
    pub value: rust_decimal::Decimal,
    pub started_at: time::PrimitiveDateTime,
    pub last_scanned_at: time::PrimitiveDateTime,
}
