use uuid::Uuid;

#[derive(Clone, Eq, PartialEq, sqlx::FromRow)]
pub struct Totp {
    pub user_id: Uuid,
    pub secret: Vec<u8>,
    pub created_at: time::PrimitiveDateTime,
}

impl core::fmt::Debug for Totp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Totp")
            .field("user_id", &self.user_id)
            .field("secret", &"[REDACTED]")
            .field("created_at", &self.created_at)
            .finish()
    }
}
