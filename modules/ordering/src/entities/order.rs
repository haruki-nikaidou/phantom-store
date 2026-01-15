use rust_decimal::Decimal;
use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct UserOrder {
    pub id: Uuid,
    pub user: Uuid,
    pub production: Uuid,
    pub total_amount: Decimal,
    pub coupon_used: Option<i32>,
    pub created_at: PrimitiveDateTime,
    pub order_status: OrderStatus,

    pub paid_at: Option<PrimitiveDateTime>,
    pub delivered_at: Option<PrimitiveDateTime>,
    pub arrived_at: Option<PrimitiveDateTime>,
    pub cancelled_at: Option<PrimitiveDateTime>,
    pub refund_requested_at: Option<PrimitiveDateTime>,
    pub refunded_at: Option<PrimitiveDateTime>,

    pub payment_method: Option<PaymentMethod>,
    pub payment_method_info: sqlx::types::Json<PaymentMethodInfo>,

    pub is_soft_deleted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, serde::Serialize)]
#[sqlx(type_name = "shop.order_status", rename_all = "snake_case")]
pub enum OrderStatus {
    Unpaid,
    Paid,
    Delivered,
    Arrived,
    Cancelled,
    Refunding,
    Refunded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, serde::Serialize)]
#[sqlx(type_name = "shop.payment_method", rename_all = "snake_case")]
pub enum PaymentMethod {
    StableCoin,
    CreditCard,
    PayPal,
    AdminOperation, // TODO
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethodInfo {
    StableCoin {
        txn_hash: String,
    },
    AdminOperation
    // reversed for credit card, paypal and others
}
