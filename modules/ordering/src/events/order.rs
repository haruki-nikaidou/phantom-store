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
pub struct OrderPaidEvent {
    pub order_id: uuid::Uuid,
    pub paid_at: i64,
}

impl framework::rabbitmq::AmqpRouting for OrderPaidEvent {
    const EXCHANGE: &'static str = "ordering";
    const EXCHANGE_TYPE: framework::rabbitmq::AmqpExchangeType =
        framework::rabbitmq::AmqpExchangeType::Direct;
    const ROUTING_KEY: &'static str = "order_paid";
}

impl framework::rabbitmq::AmqpMessageSend for OrderPaidEvent {}

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
pub struct OrderStatusChangedEvent {
    pub order_id: uuid::Uuid,
    pub new_status: crate::entities::order::OrderStatus,
    pub changed_at: i64,
}

impl framework::rabbitmq::AmqpRouting for OrderStatusChangedEvent {
    const EXCHANGE: &'static str = "ordering";
    const EXCHANGE_TYPE: framework::rabbitmq::AmqpExchangeType =
        framework::rabbitmq::AmqpExchangeType::Direct;
    const ROUTING_KEY: &'static str = "order_status_changed";
}

impl framework::rabbitmq::AmqpMessageSend for OrderStatusChangedEvent {}
