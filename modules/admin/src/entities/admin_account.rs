use compact_str::CompactString;
use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use time::PrimitiveDateTime;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct AdminAccount {
    pub id: Uuid,
    pub role: AdminRole,
    pub name: CompactString,
    pub created_at: PrimitiveDateTime,
    pub password_hash: CompactString,
    pub email: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "admin.admin_role", rename_all = "lowercase")]
pub enum AdminRole {
    Owner,
    Moderator,
}

#[derive(Debug, Clone, Copy)]
pub struct FindAdminById {
    pub id: Uuid,
}

impl Processor<FindAdminById> for DatabaseProcessor {
    type Output = Option<AdminAccount>;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:FindAdminById", err)]
    async fn process(&self, input: FindAdminById) -> Result<Option<AdminAccount>, sqlx::Error> {
        sqlx::query_as!(
            AdminAccount,
            r#"
            SELECT id, role as "role: AdminRole", name, created_at, password_hash, email, avatar
            FROM "admin"."admin_account"
            WHERE id = $1
            "#,
            input.id
        )
        .fetch_optional(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct CreateAdminAccount {
    pub role: AdminRole,
    pub password_hash: CompactString,
    pub email: String,
    pub avatar: Option<String>,
}

impl Processor<CreateAdminAccount> for DatabaseProcessor {
    type Output = AdminAccount;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:CreateAdminAccount", err)]
    async fn process(&self, input: CreateAdminAccount) -> Result<AdminAccount, sqlx::Error> {
        sqlx::query_as!(
            AdminAccount,
            r#"
            INSERT INTO "admin"."admin_account" (role, password_hash, email, avatar) 
            VALUES ($1, $2, $3, $4)
            RETURNING
            id, role as "role: AdminRole", name, created_at, email, avatar, password_hash
            "#,
            input.role as AdminRole,
            &input.password_hash,
            &input.email,
            input.avatar
        )
        .fetch_one(self.db())
        .await
    }
}
