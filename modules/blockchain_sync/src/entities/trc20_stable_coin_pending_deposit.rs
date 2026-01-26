use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;

use crate::utils::supported_tokens::StableCoinName;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Trc20StablecoinPendingDeposit {
    pub id: i64,
    pub token_name: StableCoinName,
    pub user_address: Option<String>,
    pub wallet_address: String,
    pub value: rust_decimal::Decimal,
    pub started_at: time::PrimitiveDateTime,
    pub last_scanned_at: time::PrimitiveDateTime,
}

#[derive(Debug, Clone)]
pub struct CreateTrc20StablecoinPendingDeposit {
    pub token_name: StableCoinName,
    pub user_address: Option<String>,
    pub wallet_address: String,
    pub value: rust_decimal::Decimal,
}

impl Processor<CreateTrc20StablecoinPendingDeposit> for DatabaseProcessor {
    type Output = Trc20StablecoinPendingDeposit;
    type Error = sqlx::Error;

    async fn process(
        &self,
        input: CreateTrc20StablecoinPendingDeposit,
    ) -> Result<Trc20StablecoinPendingDeposit, sqlx::Error> {
        sqlx::query_as!(
            Trc20StablecoinPendingDeposit,
            r#"
            INSERT INTO "blockchain"."trc20_stablecoin_pending_deposit" (token_name, user_address, wallet_address, value)
            VALUES ($1, $2, $3, $4)
            RETURNING id, token_name as "token_name: StableCoinName", user_address, wallet_address, value, started_at, last_scanned_at
            "#,
            input.token_name as StableCoinName,
            input.user_address,
            input.wallet_address,
            input.value
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct FindTrc20DepositByWalletAddress {
    pub wallet_address: String,
    pub token_name: StableCoinName,
    pub user_address: Option<String>,
}

impl Processor<FindTrc20DepositByWalletAddress> for DatabaseProcessor {
    type Output = Vec<Trc20StablecoinPendingDeposit>;
    type Error = sqlx::Error;

    async fn process(
        &self,
        input: FindTrc20DepositByWalletAddress,
    ) -> Result<Vec<Trc20StablecoinPendingDeposit>, sqlx::Error> {
        sqlx::query_as!(
            Trc20StablecoinPendingDeposit,
            r#"
            SELECT id, token_name as "token_name: StableCoinName", user_address, wallet_address, value, started_at, last_scanned_at
            FROM "blockchain"."trc20_stablecoin_pending_deposit"
            WHERE wallet_address = $1 AND token_name = $2 AND user_address = $3
            "#,
            input.wallet_address,
            input.token_name as StableCoinName,
            input.user_address
        )
        .fetch_all(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct UpdateTrc20StablecoinPendingDeposit {
    pub id: i64,
    pub last_scanned_at: time::PrimitiveDateTime,
}

impl Processor<UpdateTrc20StablecoinPendingDeposit> for DatabaseProcessor {
    type Output = Trc20StablecoinPendingDeposit;
    type Error = sqlx::Error;
    async fn process(
        &self,
        input: UpdateTrc20StablecoinPendingDeposit,
    ) -> Result<Trc20StablecoinPendingDeposit, sqlx::Error> {
        sqlx::query_as!(
            Trc20StablecoinPendingDeposit,
            r#"
            UPDATE "blockchain"."trc20_stablecoin_pending_deposit"
            SET last_scanned_at = $1
            WHERE id = $2
            RETURNING id, token_name as "token_name: StableCoinName", user_address, wallet_address, value, started_at, last_scanned_at
            "#,
            input.last_scanned_at,
            input.id
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct DeleteTrc20StablecoinPendingDeposit {
    pub id: i64,
}

impl Processor<DeleteTrc20StablecoinPendingDeposit> for DatabaseProcessor {
    type Output = ();
    type Error = sqlx::Error;
    async fn process(&self, input: DeleteTrc20StablecoinPendingDeposit) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM "blockchain"."trc20_stablecoin_pending_deposit"
            WHERE id = $1
            "#,
            input.id
        )
        .execute(self.db())
        .await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DeleteTrc20DepositBefore {
    pub before: time::PrimitiveDateTime,
}

impl Processor<DeleteTrc20DepositBefore> for DatabaseProcessor {
    type Output = ();
    type Error = sqlx::Error;
    async fn process(&self, input: DeleteTrc20DepositBefore) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM "blockchain"."trc20_stablecoin_pending_deposit"
            WHERE started_at < $1
            "#,
            input.before
        )
        .execute(self.db())
        .await?;
        Ok(())
    }
}
