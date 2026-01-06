use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, sqlx::FromRow)]
pub struct UserAccount {
    pub id: Uuid,
    pub name: Option<String>,
    pub email: String,
    pub created_at: time::PrimitiveDateTime,
    pub updated_at: time::PrimitiveDateTime,
}

#[derive(Debug, Clone)]
pub struct FindUserAccountByEmail {
    pub email: String,
}

impl Processor<FindUserAccountByEmail, Result<Option<UserAccount>, sqlx::Error>>
    for DatabaseProcessor
{
    #[instrument(skip_all, name = "SQL:FindUserAccountByEmail", err)]
    async fn process(
        &self,
        input: FindUserAccountByEmail,
    ) -> Result<Option<UserAccount>, sqlx::Error> {
        sqlx::query_as!(
            UserAccount,
            r#"
            SELECT id, name, email, created_at, updated_at
            FROM "auth"."user_account"
            WHERE email = $1
            "#,
            &input.email
        )
        .fetch_optional(self.db())
        .await
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FindUserAccountById {
    pub id: Uuid,
}

impl Processor<FindUserAccountById, Result<Option<UserAccount>, sqlx::Error>>
    for DatabaseProcessor
{
    #[instrument(skip_all, name = "SQL:FindUserAccountById", err)]
    async fn process(
        &self,
        input: FindUserAccountById,
    ) -> Result<Option<UserAccount>, sqlx::Error> {
        sqlx::query_as!(
            UserAccount,
            r#"
            SELECT id, name, email, created_at, updated_at
            FROM "auth"."user_account"
            WHERE id = $1
            "#,
            input.id
        )
        .fetch_optional(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct UpdateUserEmail {
    pub id: Uuid,
    pub email: String,
}

impl Processor<UpdateUserEmail, Result<UserAccount, sqlx::Error>> for DatabaseProcessor {
    #[instrument(skip_all, name = "SQL:UpdateUserEmail", err)]
    async fn process(&self, input: UpdateUserEmail) -> Result<UserAccount, sqlx::Error> {
        sqlx::query_as!(
            UserAccount,
            r#"
            UPDATE "auth"."user_account"
            SET email = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, email, created_at, updated_at
            "#,
            input.id,
            &input.email
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct UpdateUserName {
    pub id: Uuid,
    pub name: Option<String>,
}

impl Processor<UpdateUserName, Result<UserAccount, sqlx::Error>> for DatabaseProcessor {
    #[instrument(skip_all, name = "SQL:UpdateUserName", err)]
    async fn process(&self, input: UpdateUserName) -> Result<UserAccount, sqlx::Error> {
        sqlx::query_as!(
            UserAccount,
            r#"
            UPDATE "auth"."user_account"
            SET name = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, email, created_at, updated_at
            "#,
            input.id,
            input.name
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct RegisterPasswordlessUserAccount {
    pub email: String,
    pub name: Option<String>,
}

impl Processor<RegisterPasswordlessUserAccount, Result<UserAccount, sqlx::Error>>
    for DatabaseProcessor
{
    #[instrument(skip_all, name = "SQL:RegisterPasswordlessUserAccount", err)]
    async fn process(
        &self,
        input: RegisterPasswordlessUserAccount,
    ) -> Result<UserAccount, sqlx::Error> {
        sqlx::query_as!(
            UserAccount,
            r#"
            INSERT INTO "auth"."user_account" (email, name)
            VALUES ($1, $2)
            RETURNING id, name, email, created_at, updated_at
            "#,
            &input.email,
            input.name
        )
        .fetch_one(self.db())
        .await
    }
}
