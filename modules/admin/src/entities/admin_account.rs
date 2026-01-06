use compact_str::CompactString;
use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct AdminAccount {
    pub id: Uuid,
    pub role: AdminRole,
    pub name: CompactString,
    pub created_at: PrimitiveDateTime,
    pub password_hash: CompactString,
    pub email: String,
    pub avatar: Option<CompactString>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "admin.admin_role", rename_all = "lowercase")]
pub enum AdminRole {
    Owner,
    Moderator,
}
