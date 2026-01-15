use crate::entities::order::PaymentMethodInfo;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    kanau::RkyvMessageSer,
    kanau::RkyvMessageDe,
)]
pub struct PaymentCallbackEvent {
    pub order_id: uuid::Uuid,
    pub payment_method_info: PaymentMethodInfo,
    pub created_at: i64,
}

impl framework::rabbitmq::AmqpRouting for PaymentCallbackEvent {
    const EXCHANGE: &'static str = "ordering";
    const EXCHANGE_TYPE: framework::rabbitmq::AmqpExchangeType =
        framework::rabbitmq::AmqpExchangeType::Direct;
    const ROUTING_KEY: &'static str = "payment_callback";
}
