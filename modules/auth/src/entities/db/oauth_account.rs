use crate::utils::oauth::providers::OAuthProviderName;
use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
/// The OAuth account of a user.
///
/// One user can have multiple OAuth accounts.
pub struct OAuthAccount {
    pub id: i64,
    pub user_id: Uuid,
    pub provider_name: OAuthProviderName,
    pub provider_user_id: String,
    pub registered_at: PrimitiveDateTime,
    pub token_updated_at: PrimitiveDateTime,
}
