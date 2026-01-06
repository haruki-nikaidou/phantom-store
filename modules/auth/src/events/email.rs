use crate::entities::db::email_otp::EmailOtpUsage;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    kanau::RkyvMessageDe,
    kanau::RkyvMessageSer,
)]
pub struct OtpEmailSendCall {
    pub email_address: String,
    pub otp_code: String,
    pub otp_usage: EmailOtpUsage,
    pub expire_after: std::time::Duration,
    pub sent_at: u64,
}

impl framework::rabbitmq::AmqpRouting for OtpEmailSendCall {
    const EXCHANGE: &'static str = "auth";
    const EXCHANGE_TYPE: framework::rabbitmq::AmqpExchangeType =
        framework::rabbitmq::AmqpExchangeType::Direct;
    const ROUTING_KEY: &'static str = "otp";
}

impl framework::rabbitmq::AmqpMessageSend for OtpEmailSendCall {}
