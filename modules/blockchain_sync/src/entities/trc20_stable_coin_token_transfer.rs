use crate::utils::supported_tokens::StableCoinName;

pub struct Trc20StableCoinTokenTransfer {
    pub id: i64,
    pub token_name: StableCoinName,
    pub from_address: String,
    pub to_address: String,
    pub txn_hash: String,
    pub value: rust_decimal::Decimal,
    pub block_number: u64,
    pub block_timestamp: time::PrimitiveDateTime,
    pub confirmed: bool,
}
