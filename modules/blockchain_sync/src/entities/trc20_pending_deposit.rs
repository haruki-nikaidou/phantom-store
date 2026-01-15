use crate::utils::supported_tokens::StableCoinName;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Trc20PendingDeposit {
    pub id: i64,
    pub token_name: StableCoinName,
    pub user_address: Option<String>,
    pub value: rust_decimal::Decimal,
    pub started_at: time::PrimitiveDateTime,
    pub last_scanned_at: time::PrimitiveDateTime,
}