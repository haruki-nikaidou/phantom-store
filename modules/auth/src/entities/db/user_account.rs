use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, sqlx::FromRow)]
pub struct UserAccount {
    pub id: Uuid,
    pub name: Option<String>,
    pub email: String,
    pub created_at: time::PrimitiveDateTime,
    pub updated_at: time::PrimitiveDateTime,
}
