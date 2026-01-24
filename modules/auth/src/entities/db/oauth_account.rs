use crate::utils::oauth::providers::OAuthProviderName;
use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;
use time::PrimitiveDateTime;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
/// The OAuth account of a user.
///
/// One user can have multiple OAuth accounts.
pub struct OAuthAccount {
    pub id: i64,
    pub user_id: Uuid,
    pub provider_name: OAuthProviderName,
    pub provider_user_id: String,
    pub registered_at: PrimitiveDateTime,
    pub token_updated_at: PrimitiveDateTime,
}

#[derive(Debug, Clone)]
pub struct FindOAuthAccountByProviderUserId {
    pub provider_name: OAuthProviderName,
    pub provider_user_id: String,
}

impl Processor<FindOAuthAccountByProviderUserId> for DatabaseProcessor {
    type Output = Option<OAuthAccount>;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:FindOAuthAccountByProviderUserId", err)]
    async fn process(
        &self,
        input: FindOAuthAccountByProviderUserId,
    ) -> Result<Option<OAuthAccount>, sqlx::Error> {
        sqlx::query_as!(
            OAuthAccount,
            r#"
            SELECT id, user_id, provider_name as "provider_name: OAuthProviderName", provider_user_id, registered_at, token_updated_at
            FROM "auth"."oauth_account"
            WHERE provider_name = $1 AND provider_user_id = $2
            "#,
            input.provider_name as OAuthProviderName,
            &input.provider_user_id
        )
        .fetch_optional(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct RegisterOAuthAccount {
    pub provider_name: OAuthProviderName,
    pub provider_user_id: String,
    pub email: String,
    pub name: Option<String>,
}

impl Processor<RegisterOAuthAccount> for DatabaseProcessor {
    type Output = OAuthAccount;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL-Transaction:RegisterOAuthAccount", err)]
    async fn process(&self, input: RegisterOAuthAccount) -> Result<OAuthAccount, sqlx::Error> {
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
        let oauth_account = sqlx::query_as!(
            OAuthAccount,
            r#"
            INSERT INTO "auth"."oauth_account" (user_id, provider_name, provider_user_id)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, provider_name as "provider_name: OAuthProviderName", provider_user_id, registered_at, token_updated_at
            "#,
            user_account.id,
            input.provider_name as OAuthProviderName,
            &input.provider_user_id
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit()
            .instrument(info_span!("<Transaction Commit>"))
            .await?;
        Ok(oauth_account)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeleteOAuthAccountById {
    pub id: i64,
}

impl Processor<DeleteOAuthAccountById> for DatabaseProcessor {
    type Output = ();
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:DeleteOAuthAccountById", err)]
    async fn process(&self, input: DeleteOAuthAccountById) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM "auth"."oauth_account"
            WHERE id = $1
            "#,
            input.id
        )
        .execute(self.db())
        .await
        .map(|_| ())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FindOAuthAccountsByUserId {
    pub user_id: Uuid,
}

impl Processor<FindOAuthAccountsByUserId> for DatabaseProcessor {
    type Output = Vec<OAuthAccount>;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:FindOAuthAccountsByUserId", err)]
    async fn process(
        &self,
        input: FindOAuthAccountsByUserId,
    ) -> Result<Vec<OAuthAccount>, sqlx::Error> {
        sqlx::query_as!(
            OAuthAccount,
            r#"
            SELECT id, user_id, provider_name as "provider_name: OAuthProviderName", provider_user_id, registered_at, token_updated_at
            FROM "auth"."oauth_account"
            WHERE user_id = $1
            "#,
            input.user_id
        )
        .fetch_all(self.db())
        .await
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FindOAuthAccountById {
    pub id: i64,
}

impl Processor<FindOAuthAccountById> for DatabaseProcessor {
    type Output = Option<OAuthAccount>;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:FindOAuthAccountById", err)]
    async fn process(
        &self,
        input: FindOAuthAccountById,
    ) -> Result<Option<OAuthAccount>, sqlx::Error> {
        sqlx::query_as!(
            OAuthAccount,
            r#"
            SELECT id, user_id, provider_name as "provider_name: OAuthProviderName", provider_user_id, registered_at, token_updated_at
            FROM "auth"."oauth_account"
            WHERE id = $1
            "#,
            input.id
        )
        .fetch_optional(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct AppendOAuthAccount {
    pub user_id: Uuid,
    pub provider_name: OAuthProviderName,
    pub provider_user_id: String,
}

impl Processor<AppendOAuthAccount> for DatabaseProcessor {
    type Output = OAuthAccount;
    type Error = sqlx::Error;
    #[instrument(skip_all, name = "SQL:AppendOAuthAccount", err)]
    async fn process(&self, input: AppendOAuthAccount) -> Result<OAuthAccount, sqlx::Error> {
        sqlx::query_as!(
            OAuthAccount,
            r#"
            INSERT INTO "auth"."oauth_account" (user_id, provider_name, provider_user_id)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, provider_name as "provider_name: OAuthProviderName", provider_user_id, registered_at, token_updated_at
            "#,
            input.user_id,
            input.provider_name as OAuthProviderName,
            &input.provider_user_id
        )
        .fetch_one(self.db())
        .await
    }
}
