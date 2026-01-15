use crate::entities::delivery_tracking::DeliveryStatus;

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct DeliveryUpdateItem {
    pub status: DeliveryStatus,
    pub location: Option<String>,
    pub description: String,
    pub created_at: i64,
}

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
pub struct DeliveryUpdate {
    pub order_id: uuid::Uuid,
    pub items: Vec<DeliveryUpdateItem>,
}

impl framework::rabbitmq::AmqpRouting for DeliveryUpdate {
    const EXCHANGE: &'static str = "ordering";
    const EXCHANGE_TYPE: framework::rabbitmq::AmqpExchangeType =
        framework::rabbitmq::AmqpExchangeType::Direct;
    const ROUTING_KEY: &'static str = "delivery_update";
}

impl framework::rabbitmq::AmqpMessageSend for DeliveryUpdate {}
