use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use tracing::{Instrument, info_span, instrument};
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

#[derive(Debug, Clone, Copy)]
pub struct FindUserPasswordByUserId {
    pub user_id: Uuid,
}

impl Processor<FindUserPasswordByUserId> for DatabaseProcessor {
    type Output = Option<UserPassword>;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:FindUserPasswordByUserId", err)]
    async fn process(
        &self,
        input: FindUserPasswordByUserId,
    ) -> Result<Option<UserPassword>, sqlx::Error> {
        sqlx::query_as!(
            UserPassword,
            r#"
            SELECT user_id, password_hash, created_at, updated_at
            FROM "auth"."user_password"
            WHERE user_id = $1
            "#,
            input.user_id
        )
        .fetch_optional(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct AppendUserPassword {
    pub user_id: Uuid,
    pub password_hash: String,
}

impl Processor<AppendUserPassword> for DatabaseProcessor {
    type Output = UserPassword;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:AppendUserPassword", err)]
    async fn process(&self, input: AppendUserPassword) -> Result<UserPassword, sqlx::Error> {
        sqlx::query_as!(
            UserPassword,
            r#"
            INSERT INTO "auth"."user_password" (user_id, password_hash)
            VALUES ($1, $2)
            RETURNING user_id, password_hash, created_at, updated_at
            "#,
            input.user_id,
            &input.password_hash
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct UpdateUserPassword {
    pub user_id: Uuid,
    pub password_hash: String,
}

impl Processor<UpdateUserPassword> for DatabaseProcessor {
    type Output = UserPassword;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:UpdateUserPassword", err)]
    async fn process(&self, input: UpdateUserPassword) -> Result<UserPassword, sqlx::Error> {
        sqlx::query_as!(
            UserPassword,
            r#"
            UPDATE "auth"."user_password"
            SET password_hash = $2, updated_at = NOW()
            WHERE user_id = $1
            RETURNING user_id, password_hash, created_at, updated_at
            "#,
            input.user_id,
            &input.password_hash
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct RegisterUserWithPassword {
    pub email: String,
    pub name: Option<String>,
    pub password_hash: String,
}

impl Processor<RegisterUserWithPassword> for DatabaseProcessor {
    type Output = UserPassword;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL-Transaction:RegisterUserWithPassword", err)]
    async fn process(&self, input: RegisterUserWithPassword) -> Result<UserPassword, sqlx::Error> {
        let mut tx = self
            .db()
            .begin()
            .instrument(info_span!("<Transaction Begin>"))
            .await?;
        let user_account = sqlx::query_as!(
            crate::entities::db::user_account::UserAccount,
            r#"
            INSERT INTO "auth"."user_account" (email, name)
            VALUES ($1, $2)
            RETURNING id, name, email, created_at, updated_at
            "#,
            &input.email,
            input.name
        )
        .fetch_one(&mut *tx)
        .await?;
        let user_password = sqlx::query_as!(
            UserPassword,
            r#"
            INSERT INTO "auth"."user_password" (user_id, password_hash)
            VALUES ($1, $2)
            RETURNING user_id, password_hash, created_at, updated_at
            "#,
            user_account.id,
            &input.password_hash
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit()
            .instrument(info_span!("<Transaction Commit>"))
            .await?;
        Ok(user_password)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeleteUserPasswordByUserId {
    pub user_id: Uuid,
}

impl Processor<DeleteUserPasswordByUserId> for DatabaseProcessor {
    type Output = ();
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:DeleteUserPasswordByUserId", err)]
    async fn process(&self, input: DeleteUserPasswordByUserId) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM "auth"."user_password"
            WHERE user_id = $1
            "#,
            input.user_id
        )
        .execute(self.db())
        .await
        .map(|_| ())
    }
}

#[derive(Debug, Clone)]
pub struct FindUserPasswordByEmail {
    pub email: String,
}

impl Processor<FindUserPasswordByEmail> for DatabaseProcessor {
    type Output = Option<UserPassword>;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:FindUserPasswordByEmail", err)]
    async fn process(&self, input: FindUserPasswordByEmail) -> Result<Option<UserPassword>, sqlx::Error> {
        sqlx::query_as!(
            UserPassword,
            r#"
            SELECT user_id, password_hash, created_at, updated_at
            FROM "auth"."user_password"
            WHERE user_id = (SELECT id FROM "auth"."user_account" WHERE email = $1)
            "#,
            &input.email
        )
        .fetch_optional(self.db())
        .await
    }
}