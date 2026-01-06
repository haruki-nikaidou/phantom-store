use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, sqlx::FromRow)]
pub struct EmailOtp {
    pub id: i64,
    pub user_id: Option<Uuid>,
    pub email: String,
    pub otp_code: String,
    pub created_at: time::PrimitiveDateTime,
    pub expires_at: time::PrimitiveDateTime,
    pub has_been_used: bool,
    pub used_at: Option<time::PrimitiveDateTime>,
    pub usage: EmailOtpUsage,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[sqlx(type_name = "auth.email_otp_usage", rename_all = "snake_case")]
pub enum EmailOtpUsage {
    Login,
    PasswordReset,
    ChangeEmailAddress,
    SudoMode,
}

pub fn generate_otp_code() -> String {
    use rand::Rng;
    format!("{:06}", rand::rng().random_range(0u32..1_000_000))
}

#[derive(Debug, Clone)]
pub struct CreateEmailOtp {
    pub user_id: Option<Uuid>,
    pub email: String,
    pub usage: EmailOtpUsage,
    pub expires_after: sqlx::postgres::types::PgInterval,
}

impl Processor<CreateEmailOtp, Result<EmailOtp, sqlx::Error>> for DatabaseProcessor {
    #[instrument(skip_all, name = "SQL:CreateEmailOtp", err)]
    async fn process(&self, input: CreateEmailOtp) -> Result<EmailOtp, sqlx::Error> {
        let otp_code = generate_otp_code();
        sqlx::query_as!(
            EmailOtp,
            r#"
            INSERT INTO "auth"."email_otp" (user_id, email, otp_code, expires_at, usage)
            VALUES ($1, $2, $3, NOW() + $4, $5)
            RETURNING 
            id, user_id, email, otp_code, created_at, 
            expires_at, has_been_used, used_at, 
            usage as "usage: EmailOtpUsage"
            "#,
            input.user_id,
            &input.email,
            &otp_code,
            input.expires_after,
            input.usage as EmailOtpUsage
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MarkEmailOtpAsUsed {
    pub id: i64,
}

impl Processor<MarkEmailOtpAsUsed, Result<EmailOtp, sqlx::Error>> for DatabaseProcessor {
    #[instrument(skip_all, name = "SQL:MarkEmailOtpAsUsed", err)]
    async fn process(&self, input: MarkEmailOtpAsUsed) -> Result<EmailOtp, sqlx::Error> {
        sqlx::query_as!(
            EmailOtp,
            r#"
            UPDATE "auth"."email_otp"
            SET has_been_used = TRUE, used_at = NOW()
            WHERE id = $1
            RETURNING 
            id, user_id, email, otp_code, created_at, 
            expires_at, has_been_used, used_at, 
            usage as "usage: EmailOtpUsage"
            "#,
            input.id
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct FindEmailOtpBySource {
    pub user_id: Uuid,
    pub email: String,
    pub usage: EmailOtpUsage,
}

impl Processor<FindEmailOtpBySource, Result<Vec<EmailOtp>, sqlx::Error>> for DatabaseProcessor {
    #[instrument(skip_all, name = "SQL:FindEmailOtpBySource", err)]
    async fn process(&self, input: FindEmailOtpBySource) -> Result<Vec<EmailOtp>, sqlx::Error> {
        sqlx::query_as!(
            EmailOtp,
            r#"
            SELECT 
            id, user_id, email, otp_code, created_at, 
            expires_at, has_been_used, used_at, 
            usage as "usage: EmailOtpUsage"
            FROM "auth"."email_otp"
            WHERE user_id = $1 AND email = $2 AND usage = $3
            "#,
            input.user_id,
            &input.email,
            input.usage as EmailOtpUsage
        )
        .fetch_all(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct FindValidEmailOtp {
    pub user_id: Uuid,
    pub email: String,
    pub usage: EmailOtpUsage,
    pub otp_code: String,
}

impl Processor<FindValidEmailOtp, Result<Option<EmailOtp>, sqlx::Error>> for DatabaseProcessor {
    #[instrument(skip_all, name = "SQL:FindValidEmailOtp", err)]
    async fn process(&self, input: FindValidEmailOtp) -> Result<Option<EmailOtp>, sqlx::Error> {
        sqlx::query_as!(
            EmailOtp,
            r#"
            SELECT 
            id, user_id, email, otp_code, created_at, 
            expires_at, has_been_used, used_at, 
            usage as "usage: EmailOtpUsage"
            FROM "auth"."email_otp"
            WHERE user_id = $1 AND email = $2 AND usage = $3 AND otp_code = $4
            AND expires_at > NOW() AND has_been_used = FALSE
            "#,
            input.user_id,
            &input.email,
            input.usage as EmailOtpUsage,
            &input.otp_code
        )
        .fetch_optional(self.db())
        .await
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeleteEmailOtpBefore {
    pub before: time::PrimitiveDateTime,
}

impl Processor<DeleteEmailOtpBefore, Result<(), sqlx::Error>> for DatabaseProcessor {
    #[instrument(skip_all, name = "SQL:DeleteEmailOtpBefore", err)]
    async fn process(&self, input: DeleteEmailOtpBefore) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM "auth"."email_otp"
            WHERE expires_at < $1
            "#,
            input.before
        )
        .execute(self.db())
        .await
        .map(|_| ())
    }
}

#[derive(Debug, Clone)]
pub struct CheckEmailFrequency {
    pub email: String,
    pub before: time::PrimitiveDateTime,
}

impl Processor<CheckEmailFrequency, Result<i64, sqlx::Error>> for DatabaseProcessor {
    #[instrument(skip_all, name = "SQL:CheckEmailFrequency", err)]
    async fn process(&self, input: CheckEmailFrequency) -> Result<i64, sqlx::Error> {
        sqlx::query!(
            r#"
            SELECT COUNT(*) FROM "auth"."email_otp"
            WHERE email = $1 AND created_at > $2
            "#,
            &input.email,
            input.before
        )
        .fetch_one(self.db())
        .await
        .map(|row| row.count.unwrap_or(0))
    }
}
