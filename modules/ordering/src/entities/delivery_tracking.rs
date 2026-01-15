use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct DeliveryTracking {
    pub id: i64,
    pub order_id: Uuid,
    pub status: DeliveryStatus,
    pub location: Option<String>,
    pub description: String,
    pub created_at: PrimitiveDateTime,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    sqlx::Type,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[sqlx(type_name = "shop.delivery_status", rename_all = "snake_case")]
pub enum DeliveryStatus {
    InTransit,
    OutForDelivery,
    Delivered,
    Cancelled,
    Returned,
}
