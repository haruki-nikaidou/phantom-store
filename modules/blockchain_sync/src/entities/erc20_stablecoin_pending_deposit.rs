use framework::sqlx::DatabaseProcessor;
use kanau::processor::Processor;

use crate::services::etherscan::EtherScanChain;
use crate::utils::supported_tokens::StableCoinName;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Erc20StablecoinPendingDeposit {
    pub id: i64,
    pub token_name: StableCoinName,
    pub chain: EtherScanChain,
    pub user_address: Option<String>,
    pub wallet_address: String,
    pub value: rust_decimal::Decimal,
    pub started_at: time::PrimitiveDateTime,
    pub last_scanned_at: time::PrimitiveDateTime,
}

#[derive(Debug, Clone)]
pub struct CreateErc20StablecoinPendingDeposit {
    pub token_name: StableCoinName,
    pub chain: EtherScanChain,
    pub user_address: Option<String>,
    pub wallet_address: String,
    pub value: rust_decimal::Decimal,
}

impl Processor<CreateErc20StablecoinPendingDeposit> for DatabaseProcessor {
    type Output = Erc20StablecoinPendingDeposit;

    type Error = sqlx::Error;

    async fn process(
        &self,
        input: CreateErc20StablecoinPendingDeposit,
    ) -> Result<Erc20StablecoinPendingDeposit, sqlx::Error> {
        sqlx::query_as!(
            Erc20StablecoinPendingDeposit,
            r#"
            INSERT INTO "blockchain"."erc20_stablecoin_pending_deposit" (token_name, chain, user_address, wallet_address, value)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, token_name as "token_name: StableCoinName", chain as "chain: EtherScanChain", user_address, wallet_address, value, started_at, last_scanned_at
            "#,
            input.token_name as StableCoinName,
            input.chain as EtherScanChain,
            input.user_address,
            input.wallet_address,
            input.value
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct FindErc20DepositByWalletAddress {
    pub wallet_address: String,
    pub token_name: StableCoinName,
    pub chain: EtherScanChain,
    pub user_address: Option<String>,
}

impl Processor<FindErc20DepositByWalletAddress> for DatabaseProcessor {
    type Output = Vec<Erc20StablecoinPendingDeposit>;
    type Error = sqlx::Error;

    async fn process(
        &self,
        input: FindErc20DepositByWalletAddress,
    ) -> Result<Vec<Erc20StablecoinPendingDeposit>, sqlx::Error> {
        sqlx::query_as!(
            Erc20StablecoinPendingDeposit,
            r#"
            SELECT id, token_name as "token_name: StableCoinName", chain as "chain: EtherScanChain", user_address, wallet_address, value, started_at, last_scanned_at
            FROM "blockchain"."erc20_stablecoin_pending_deposit"
            WHERE wallet_address = $1 AND token_name = $2 AND chain = $3 AND user_address = $4
            "#,
            input.wallet_address,
            input.token_name as StableCoinName,
            input.chain as EtherScanChain,
            input.user_address
        )
        .fetch_all(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct UpdateErc20StablecoinPendingDeposit {
    pub id: i64,
    pub last_scanned_at: time::PrimitiveDateTime,
}

impl Processor<UpdateErc20StablecoinPendingDeposit> for DatabaseProcessor {
    type Output = Erc20StablecoinPendingDeposit;
    type Error = sqlx::Error;

    async fn process(
        &self,
        input: UpdateErc20StablecoinPendingDeposit,
    ) -> Result<Erc20StablecoinPendingDeposit, sqlx::Error> {
        sqlx::query_as!(
            Erc20StablecoinPendingDeposit,
            r#"
            UPDATE "blockchain"."erc20_stablecoin_pending_deposit"
            SET last_scanned_at = $1
            WHERE id = $2
            RETURNING id, token_name as "token_name: StableCoinName", chain as "chain: EtherScanChain", user_address, wallet_address, value, started_at, last_scanned_at
            "#,
            input.last_scanned_at,
            input.id
        )
        .fetch_one(self.db())
        .await
    }
}

#[derive(Debug, Clone)]
pub struct DeleteErc20StablecoinPendingDeposit {
    pub id: i64,
}

impl Processor<DeleteErc20StablecoinPendingDeposit> for DatabaseProcessor {
    type Output = ();
    type Error = sqlx::Error;

    async fn process(&self, input: DeleteErc20StablecoinPendingDeposit) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM "blockchain"."erc20_stablecoin_pending_deposit"
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
pub struct DeleteErc20DepositBefore {
    pub before: time::PrimitiveDateTime,
}

impl Processor<DeleteErc20DepositBefore> for DatabaseProcessor {
    type Output = ();
    type Error = sqlx::Error;

    async fn process(&self, input: DeleteErc20DepositBefore) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM "blockchain"."erc20_stablecoin_pending_deposit"
            WHERE started_at < $1
            "#,
            input.before
        )
        .execute(self.db())
        .await?;
        Ok(())
    }
}
