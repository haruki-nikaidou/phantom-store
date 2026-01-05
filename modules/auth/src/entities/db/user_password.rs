use uuid::Uuid;

#[derive(Clone, Eq, PartialEq, sqlx::FromRow)]
pub struct UserPassword {
    pub user_id: Uuid,
    pub password_hash: String,
    pub created_at: time::PrimitiveDateTime,
    pub updated_at: time::PrimitiveDateTime,
}

impl core::fmt::Debug for UserPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserPassword")
            .field("user_id", &self.user_id)
            .field("password_hash", &"[REDACTED]")
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}
