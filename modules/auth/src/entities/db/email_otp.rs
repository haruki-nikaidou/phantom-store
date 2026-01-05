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
