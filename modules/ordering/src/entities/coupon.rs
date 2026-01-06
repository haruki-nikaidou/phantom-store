use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Coupon {
    pub id: i32,
    pub code: String,
    pub set_active: bool,
    pub discount: Discount,
    pub available_since: Option<time::PrimitiveDateTime>,
    pub available_until: Option<time::PrimitiveDateTime>,
    pub limit_to_category: Option<i32>,
    pub limit_per_user: Option<i32>,
    pub limit_total: Option<i32>,
    pub used_count: i32,
    pub created_at: time::PrimitiveDateTime,
    pub updated_at: time::PrimitiveDateTime,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum Discount {
    Rate(RateDiscount),
    Amount(AmountDiscount),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct RateDiscount {
    pub rate: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct AmountDiscount {
    pub min_amount: Decimal,
    pub discount: Decimal,
}
