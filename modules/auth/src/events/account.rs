use crate::utils::oauth::providers::OAuthProviderName;
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum RegisterMethod {
    EmailAccount {
        user_id: Uuid,
        has_password: bool,
    },
    OAuth {
        user_id: Uuid,
        oauth_account_id: i64,
        provider: OAuthProviderName,
        access_token: Option<String>,
        refresh_token: Option<String>,
    },
}

impl core::fmt::Debug for RegisterMethod {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RegisterMethod::EmailAccount {
                user_id,
                has_password,
            } => f
                .debug_struct("EmailAccount")
                .field("user_id", user_id)
                .field("has_password", has_password)
                .finish(),
            RegisterMethod::OAuth {
                user_id,
                oauth_account_id,
                provider,
                access_token,
                refresh_token,
            } => f
                .debug_struct("OAuth")
                .field("user_id", user_id)
                .field("oauth_account_id", oauth_account_id)
                .field("provider", provider)
                .field(
                    "access_token",
                    match access_token {
                        Some(_) => &"Some([REDACTED])",
                        None => &"None",
                    },
                )
                .field(
                    "refresh_token",
                    match refresh_token {
                        Some(_) => &"Some([REDACTED])",
                        None => &"None",
                    },
                )
                .finish(),
        }
    }
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
pub struct UserRegisterEvent {
    pub user_id: Uuid,
    pub registered_at: u64,
    pub register_method: RegisterMethod,
    pub register_with_order_creation: bool,
}

impl framework::rabbitmq::AmqpRouting for UserRegisterEvent {
    const EXCHANGE: &'static str = "auth";
    const EXCHANGE_TYPE: framework::rabbitmq::AmqpExchangeType =
        framework::rabbitmq::AmqpExchangeType::Direct;
    const ROUTING_KEY: &'static str = "user_register";
}

impl framework::rabbitmq::AmqpMessageSend for UserRegisterEvent {}
