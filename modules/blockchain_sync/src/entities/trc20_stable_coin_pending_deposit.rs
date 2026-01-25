use crate::utils::supported_tokens::StableCoinName;

pub struct Trc20StablecoinPendingDeposit {
    pub id: i64,
    pub token_name: StableCoinName,
    pub user_address: Option<String>,
    pub wallet_address: String,
    pub value: rust_decimal::Decimal,
    pub started_at: time::PrimitiveDateTime,
    pub last_scanned_at: time::PrimitiveDateTime,
}
