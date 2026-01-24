use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use tracing::instrument;
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

#[derive(Debug, Clone, Copy)]
pub struct FindTotpByUserId {
    pub user_id: Uuid,
}

impl Processor<FindTotpByUserId> for DatabaseProcessor {
    type Output = Option<Totp>;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:FindTotpByUserId", err)]
    async fn process(&self, input: FindTotpByUserId) -> Result<Option<Totp>, sqlx::Error> {
        sqlx::query_as!(
            Totp,
            r#"
            SELECT user_id, secret, created_at
            FROM "auth"."totp"
            WHERE user_id = $1
            "#,
            input.user_id
        )
        .fetch_optional(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct CreateTotp {
    pub user_id: Uuid,
    pub secret: Vec<u8>,
}

impl Processor<CreateTotp> for DatabaseProcessor {
    type Output = Totp;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:CreateTotp", err)]
    async fn process(&self, input: CreateTotp) -> Result<Totp, sqlx::Error> {
        sqlx::query_as!(
            Totp,
            r#"
            INSERT INTO "auth"."totp" (user_id, secret)
            VALUES ($1, $2)
            RETURNING user_id, secret, created_at
            "#,
            input.user_id,
            input.secret
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RemoveTotpByUserId {
    pub user_id: Uuid,
}

impl Processor<RemoveTotpByUserId> for DatabaseProcessor {
    type Output = ();
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:RemoveTotpByUserId", err)]
    async fn process(&self, input: RemoveTotpByUserId) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM "auth"."totp"
            WHERE user_id = $1
            "#,
            input.user_id
        )
        .execute(self.db())
        .await
        .map(|_| ())
    }
}
