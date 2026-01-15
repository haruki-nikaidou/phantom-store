use crate::entities::order::{PaymentMethod, PaymentMethodInfo};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct PaymentCallback {
    pub id: i64,
    pub order_id: Uuid,
    pub payment_method: PaymentMethod,
    pub payment_method_info: sqlx::types::Json<PaymentMethodInfo>,
    pub created_at: time::PrimitiveDateTime,
    pub checked_at: Option<time::PrimitiveDateTime>,
}
